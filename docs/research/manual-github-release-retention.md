# Manual GitHub release and artifact retention

## Recommendation

Use one manually dispatched workflow for the complete release transaction:

1. Accept and validate a SemVer `version` input and derive `v${version}`.
2. Require the dispatch to run from the default branch and bind the tag to the dispatched `github.sha`.
3. Create the GitHub Release as a draft.
4. Build every platform package, upload the packages, generate attestations, and upload `SHA256SUMS.txt`.
5. Publish the release only after every required job succeeds.
6. After publication succeeds, keep the uploaded assets on the newest two published releases and remove uploaded assets from older releases.

This keeps version selection operator-controlled without requiring a release PR, manifest, version-file mutation, custom token, or second workflow handoff.

GitHub exposes a manual workflow in the Actions UI when it uses `workflow_dispatch`. The workflow file must be present on the default branch, and the person running it needs repository write access. The same workflow can be invoked with GitHub CLI or the REST API. [GitHub: manually running a workflow](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)

Keep creation, build, upload, publication, and cleanup in the same workflow. Events produced with the repository's `GITHUB_TOKEN` do not start another workflow, except for `workflow_dispatch` and `repository_dispatch`; therefore a release created by one job should not be expected to activate a separate `release: published` workflow. [GitHub: `GITHUB_TOKEN` event behavior](https://docs.github.com/en/actions/concepts/security/github_token#when-github_token-triggers-workflow-runs)

Use the built-in `GITHUB_TOKEN`. Give only jobs that create, update, upload to, or prune releases `contents: write`; GitHub documents that permission as sufficient to create a release. Preserve the existing `id-token: write`, `attestations: write`, and `artifact-metadata: write` permissions on the package attestation job. [GitHub: workflow permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#permissions)

The draft-first sequence is useful even without release immutability: users never see a partially populated release, and a failed matrix leaves a draft rather than a published release missing platforms or checksums. It is also GitHub's recommended shape if immutable releases are enabled later: create a draft, attach every asset, then publish. [GitHub: immutable release best practices](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases#best-practices-for-publishing-immutable-releases)

The existing Tauri action remains a good fit. Its first-party documentation supports both creating a release with application bundles and uploading bundles to an existing release, and requires `releaseDraft: true` when the target release is a draft. [Tauri action documentation](https://github.com/tauri-apps/tauri-action)

## What “artifacts” means in this repository

There are two different GitHub storage types:

- `.github/workflows/release.yml` sends installers and `SHA256SUMS.txt` directly to a GitHub Release. These are **release assets**, not Actions workflow artifacts. `tauri-action` only creates Actions artifacts when `uploadWorkflowArtifacts` is enabled, and this workflow does not enable it. [Tauri action configuration](https://github.com/tauri-apps/tauri-action#usage)
- `.github/workflows/windows-build-smoke.yml` uses `actions/upload-artifact`, so its two matrix outputs are **Actions workflow artifacts**.

The cleanup is a no-op while no more than two published releases have uploaded assets. Publishing a third release removes the uploaded assets from the oldest release.

## Release asset cleanup

The preferred policy is:

- Count all published releases, including prereleases, by publication time.
- Exclude drafts because they are incomplete work, not shipped versions.
- Keep every uploaded asset on the newest release and one previous release.
- Delete uploaded assets from every older release while preserving its tag, notes, and historical record.
- Run cleanup only after all builds, attestations, checksums, and any requested publication have succeeded.

GitHub CLI can list releases in descending order and expose `tagName`, `publishedAt`, draft, prerelease, and immutability fields. [GitHub CLI: `gh release list`](https://cli.github.com/manual/gh_release_list)

An Ubuntu-runner implementation can use the following shape:

```bash
mapfile -t releases < <(
  gh release list \
    --exclude-drafts \
    --limit 1000 \
    --order desc \
    --json tagName,publishedAt \
    --jq 'sort_by(.publishedAt) | reverse | .[].tagName'
)

for tag in "${releases[@]:2}"; do
  while IFS= read -r asset; do
    gh release delete-asset "$tag" "$asset" --yes
  done < <(gh release view "$tag" --json assets --jq '.assets[].name')
done
```

Asset-scoped deletion is an official operation, available as `gh release delete-asset <tag> <asset-name>` and as `DELETE /repos/{owner}/{repo}/releases/assets/{asset_id}`. The REST operation requires `Contents: write`. Preserving the release object while deleting only its assets is an inference from that distinct asset endpoint. [GitHub CLI: delete a release asset](https://cli.github.com/manual/gh_release_delete-asset), [GitHub REST: delete a release asset](https://docs.github.com/en/rest/releases/assets#delete-a-release-asset)

Do not delete the old release object by default. `gh release delete` is a separate operation, and its optional `--cleanup-tag` flag also deletes the tag. That loses useful release history and is broader than the storage requirement. [GitHub CLI: delete a release](https://cli.github.com/manual/gh_release_delete)

Two limitations are important:

- GitHub automatically shows source ZIP and tarball links for every tagged release. They are generated from the tag and remain available even after all user-uploaded release assets are removed. Strictly removing those links requires deleting the release/tag rather than pruning assets. [GitHub: about releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- Published immutable releases prohibit release-asset deletion. If release immutability is enabled, satisfy the count policy by deleting whole old releases instead, or choose immutability over asset pruning. [GitHub: immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)

## Actions workflow artifact cleanup

The Windows smoke installers are unrelated to published product versions. Set `retention-days: 1` on its `actions/upload-artifact` step so transient installers expire at GitHub's minimum supported interval. The action's official input defines one day as the minimum; GitHub also allows retention to be configured per artifact. [GitHub `upload-artifact` input](https://github.com/actions/upload-artifact/blob/main/action.yml), [GitHub: custom artifact retention](https://docs.github.com/en/actions/tutorials/store-and-share-data#configuring-a-custom-artifact-retention-period)

Time-based retention cannot guarantee an exact maximum of two workflow runs when several runs finish within one day. If that exact count is also required for smoke artifacts, list repository artifacts with the Actions REST API, group them by `workflow_run.id`, retain the two newest run IDs, and delete every artifact belonging to older runs. The delete endpoint requires `Actions: write`. Because the smoke workflow is a two-entry matrix, retaining two artifact IDs would retain only one run, not two. [GitHub REST: Actions artifacts](https://docs.github.com/en/rest/actions/artifacts)

GitHub retains Actions artifacts and logs for 90 days by default. Repository retention may be set to 1–90 days for a public repository or 1–400 days for a private repository, and changes apply only to new artifacts. Deletion is irreversible. [GitHub: Actions artifact retention settings](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository#configuring-the-retention-period-for-github-actions-artifacts-and-logs-in-your-repository), [GitHub: removing workflow artifacts](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/remove-workflow-artifacts)

## Repository implementation

- `.github/workflows/release.yml` is `workflow_dispatch`-only and uses the built-in `GITHUB_TOKEN`.
- The workflow creates and populates a draft release, then publishes it after all packages, attestations, and checksums succeed.
- A final job prunes uploaded assets from releases older than the newest two.
- Windows smoke workflow artifacts use `retention-days: 1`.
- `CHANGELOG.md` is maintained manually, while the GitHub Release body is populated with generated release notes. `gh release create --generate-notes` generates release notes and can target an explicit branch or commit. [GitHub CLI: create a release](https://cli.github.com/manual/gh_release_create)
