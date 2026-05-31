todo:
- revisit requirements, check test suite
- introduce coverage report, CI/CD
- update README
- virtual sink should be used for recording, spotify audio redirected to it, loopback be executed in the background if user wants to hear the stream while recording
- audio files shouldn't have that small crack at the beginning of each recording
- fault tolerance: what if pulse audio sink is added while recording (I think silence), what is spotify is closed during recording? what if a second spotify is started?
- extend to record music from spotify web running in browser (chrome / firefox)
- extend to other apps like tidal or even youtube??

done:
- ~~audio recording quality needs to be configurable~~
- ~~fix warning about recording subprocess exits with 255~~
- ~~what if advertisements are played? service should handle invalid dbus metadata~~
- ~~songs should not be exported when playback in spotify stops, we need to discard those songs~~
- ~~ffmpeg output should not be visible in terminal~~
