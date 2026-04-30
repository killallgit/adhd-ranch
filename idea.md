A display to wrangle all the in-flight things. My problem is that i lose my place with these agents and the overall work being done. not sure the best solution for this. What i'm thinking is this:
- a small overlay - always on top widget showing a list of "attention" items: these are single sentence "cues" for the user to keep track of all the in-flight bullshit
- the cues have some default TTL or alert like reminders.
- the cues are updated via hooks registered to code agents
- this is a simple single sentence item is read from a basic markdown or json item and they should be able to be cleaned up in the widget with a trash button
- need to trigger system alerts. decide whether or not we structure the markdown list
- we need a good method to concisely sum up what is going on at the highest level possible.
- need a non-intrusive and easy way to do this. I guess it's the user that needs to create a new attention item but that somehow needs to have a way that agents can independently update the shared json item/remove add key points. this will happen by way of hook check-ins at milestones (TBD)

user story:
As a user i lose track of what i'm working on when i'm working on tens of microservices at once. I want to see a widget in the top right of my screen or a lilttle app bar app that has a list of the high level focuses going on. Example: working on a jira ticket or github issue xyz may or may not have a related epic and maybe spawns several other sub-tasks related to a bug where a "persistence" field isn't being processed through an api, an sdk, and this has effected a customer X. My widget should display a list and sub items in very simple terms:
- Customer X bug
	- add persistence field in compute api
	- update and test sdk
These are nested cause they're related. The user can add manual details as well
- Customer X bug
	- add persistence field in compute api
		- release staging // i added manually
		- promote production // i added manually
	- update and test sdk
		- cr checks
		- release
		- smoke test
Essentially we're distilling down jira / github issues into the most high level and manageable form so that people that have trouble keeping track dont have to wade through a tree of tickets across github or jira. I think part of the hook that updates this can watch jira and github (v2) perhaps. V1 we can just see how it feels to be able to have a kind of auto-kanban or something. 
