# Getting last heart rate from Oura

UPDATE: Something seems to have changed with the oura API that it's not returning real-time stats anymore, so this isn't very useful.

This gets the most-recent heart rate reading from Oura, displaying the beats per minute, and minutes since the last reading. Example output:

`73 | 29m`

It will grab your attention when either of these values go over the thresholds set in the code:

`>>>>> 103 <<<<< | 29m`

`73 | >>>>> 65m <<<<<`

For this to work you need to set an Oura [personal access token](https://cloud.ouraring.com/docs/authentication#personal-access-tokens) in an environment variable `OURA_ACCESS_TOKEN`.

I made this so that I could put it in the lower-right of my tmux window, and notice to take a break when necessary:

![oura-heartrate-tmux](doc/oura-heartrate-tmux.png)

