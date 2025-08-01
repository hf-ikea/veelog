# veelog
### Nix
For a nix development enviroment, clone this repository and run `nix develop` in the root. All tools needed to run `cargo run` or `cargo build` are included.
### todo
* use hashmap in adif ? works fine tbh + hashmap doesnt preserve insert order, so would have to be indexmap/similar
* add numbervalidation to prettyvalidategrid
* cleanup adif field -> veelog field type + allow for reverse for export
* adif import sucks but functional
* adif + cabrillo export

* proper settings, file dialog, pretty quickbar, etc
* add gridsquare input box + validate
* validation for all boxes, changes to different colour (red) if invalid
* actually log the qso, on enter check if relevant boxes are valid/have a valid placeholder, then log, regardless of if we are focused on the last box or not
* maybe use enter for switching boxes + then log at end?
* better tab support, always focused on a box (except in menu), maybe capture input from some other way?
* actually get which box we are/supposed to be focused on rather than trusting the value, can be desynced if tab is captured by the input box
