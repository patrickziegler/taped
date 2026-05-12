# Upfront research

## DBus related

The tool `busctl` allows to research the DBus integration of Spotify.

```sh
busctl --user tree
...
Service org.mpris.MediaPlayer2.spotify:
└─ /org
  ├─ /org/ayatana
  │ └─ /org/ayatana/NotificationItem
  │   └─ /org/ayatana/NotificationItem/spotify_client
  │     └─ /org/ayatana/NotificationItem/spotify_client/Menu
  └─ /org/mpris
    └─ /org/mpris/MediaPlayer2
```
    
```sh
busctl --user introspect org.mpris.MediaPlayer2.spotify /org/mpris/MediaPlayer2
...
NAME                                TYPE      SIGNATURE RESULT/VALUE                             FLAGS
org.freedesktop.DBus.Introspectable interface -         -                                        -
.Introspect                         method    -         s                                        -
org.freedesktop.DBus.Peer           interface -         -                                        -
.GetMachineId                       method    -         s                                        -
.Ping                               method    -         -                                        -
org.freedesktop.DBus.Properties     interface -         -                                        -
.Get                                method    ss        v                                        -
.GetAll                             method    s         a{sv}                                    -
.Set                                method    ssv       -                                        -
.PropertiesChanged                  signal    sa{sv}as  -                                        -
org.mpris.MediaPlayer2              interface -         -                                        -
.Quit                               method    -         -                                        -
.Raise                              method    -         -                                        -
.CanQuit                            property  b         true                                     emits-change
.CanRaise                           property  b         true                                     emits-change
.CanSetFullscreen                   property  b         false                                    emits-change
.DesktopEntry                       property  s         "spotify"                                emits-change
.HasTrackList                       property  b         false                                    emits-change
.Identity                           property  s         "Spotify"                                emits-change
.SupportedMimeTypes                 property  as        0                                        emits-change
.SupportedUriSchemes                property  as        1 "spotify"                              emits-change
org.mpris.MediaPlayer2.Player       interface -         -                                        -
.LoadContextUri                     method    s         -                                        -
.Next                               method    -         -                                        -
.OpenUri                            method    s         -                                        -
.Pause                              method    -         -                                        -
.Play                               method    -         -                                        -
.PlayPause                          method    -         -                                        -
.Previous                           method    -         -                                        -
.Seek                               method    x         -                                        -
.SetPosition                        method    ox        -                                        -
.Stop                               method    -         -                                        -
.CanControl                         property  b         true                                     emits-change
.CanGoNext                          property  b         true                                     emits-change
.CanGoPrevious                      property  b         true                                     emits-change
.CanPause                           property  b         true                                     emits-change
.CanPlay                            property  b         true                                     emits-change
.CanSeek                            property  b         true                                     emits-change
.LoopStatus                         property  s         "None"                                   emits-change writable
.MaximumRate                        property  d         1                                        emits-change
.Metadata                           property  a{sv}     11 "mpris:trackid" s "/com/spotify/trac… emits-change
.MinimumRate                        property  d         1                                        emits-change
.PlaybackStatus                     property  s         "Playing"                                emits-change
.Position                           property  x         55633000                                 emits-change
.Rate                               property  d         1                                        emits-change writable
.Shuffle                            property  b         false                                    emits-change writable
.Volume                             property  d         1                                        emits-change writable
.Seeked                             signal    x         -                                        -
```

```sh
busctl --user monitor --match="sender='org.mpris.MediaPlayer2.spotify',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged'"
...
‣ Type=signal  Endian=l  Flags=1  Version=1 Cookie=153  Timestamp="Tue 2026-01-27 13:11:35.043199 UTC"
  Sender=:1.78  Path=/org/mpris/MediaPlayer2  Interface=org.freedesktop.DBus.Properties  Member=PropertiesChanged
  UniqueName=:1.78
  MESSAGE "sa{sv}as" {
          STRING "org.mpris.MediaPlayer2.Player";
          ARRAY "{sv}" {
                  DICT_ENTRY "sv" {
                          STRING "Metadata";
                          VARIANT "a{sv}" {
                                  ARRAY "{sv}" {
                                          DICT_ENTRY "sv" {
                                                  STRING "mpris:trackid";
                                                  VARIANT "s" {
                                                          STRING "/com/spotify/track/3VQuZhYpXDUxawmAH4zA5u";
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "mpris:length";
                                                  VARIANT "t" {
                                                          UINT64 261000000;
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "mpris:artUrl";
                                                  VARIANT "s" {
                                                          STRING "https://i.scdn.co/image/ab67616d0000b273ee70cf81563f35af72f31fc0";
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:album";
                                                  VARIANT "s" {
                                                          STRING "Ich und meine Ubahn (Extrawelt Remixes)";
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:albumArtist";
                                                  VARIANT "as" {
                                                          ARRAY "s" {
                                                                  STRING "11Schnull";
                                                          };
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:artist";
                                                  VARIANT "as" {
                                                          ARRAY "s" {
                                                                  STRING "11Schnull";
                                                          };
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:autoRating";
                                                  VARIANT "d" {
                                                          DOUBLE 0,29;
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:discNumber";
                                                  VARIANT "i" {
                                                          INT32 1;
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:title";
                                                  VARIANT "s" {
                                                          STRING "Ich und meine Ubahn - Extrawelt Remix";
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:trackNumber";
                                                  VARIANT "i" {
                                                          INT32 1;
                                                  };
                                          };
                                          DICT_ENTRY "sv" {
                                                  STRING "xesam:url";
                                                  VARIANT "s" {
                                                          STRING "https://open.spotify.com/track/3VQuZhYpXDUxawmAH4zA5u";
                                                  };
                                          };
                                  };
                          };
                  };
          };
          ARRAY "s" {
          };
  };
```

## PipeWire related

Create a new virtual sink with

```sh
pactl load-module module-null-sink sink_name=$SINK_NAME sink_properties=device.description=SpotifyRecord >/dev/null
```

The resulting sink id can be found with

```sh
SINK_ID=$(wpctl status | grep -F "$SINK_NAME" | awk '{print $1}' | tr -d '.')
```

The Spotify PipeWire node id can be found with

```sh
NODE_ID=$(pw-dump | jq -r '
    .[]
    | select(.type=="PipeWire:Interface:Node")
    | select(.info.props["application.name"]=="spotify")
    | .id
  ' | head -n1)
```

Routing audio streams to the virtual sink can be done with

```sh

wpctl set-target "$NODE_ID" "$SINK_ID"
```

The default sink can be restored with

```sh
wpctl set-target "$NODE_ID" @DEFAULT_AUDIO_SINK@
```

Audio passthrough can be enabled with

```sh
pw-loopback \
        --capture-props="target.object=$SINK_NAME.monitor" \
        --playback-props="node.target=@DEFAULT_AUDIO_SINK@"
```

Finding the default audio sink

```sh
pactl list short sources | awk '/\.monitor/ {print $1, $2}' | grep "$(pactl get-default-sink).monitor" | awk '{print $1}'

```

Recording can be done with

```sh
pw-cat --record --target 50 --format s16 --rate 48000 --channels 2 - | ffmpeg -f s16le -ar 48000 -ac 2 -i pipe:0 output.mp3
```

## Observed system behavior

- When Spotify starts up, its interface becomes available on DBus immediately
- When Spotify shuts down, its interface on DBus disappears
- Not right after startup, but only when it starts to play a song, Spotify appears as a Node in the PipeWire graph
- Even when playback is stopped, the PipeWire node remains present until Spotify shuts down
- On playback start, and on every change of track, Spotify sends a PropertiesChanged event for Metadata of the new track on org.mpris.MediaPlayer2.Player
- The Metadata may have a field called mpris:artUrl which contains a URL to the album cover image that can be downloaded
