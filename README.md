# Getting last heart rate from Oura

This gets the most-recent heart rate reading from Oura, displaying the beats per minute, and minutes since the last reading. Example output:

`73 | 29m`

It will grab your attention when either of these values go over the thresholds set in the code:

`>>>>> 103 <<<<< | 29m`

`73 | >>>>> 65m <<<<<`

For this to work you need to set an Oura [personal access token](https://cloud.ouraring.com/docs/authentication#personal-access-tokens) in an environment variable `OURA_ACCESS_TOKEN`.

