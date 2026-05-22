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

The following capture shows the typical metadata information as propagated by Spotify via DBus

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

The following capture shows a session with advertisements played in between

```sh
patrick@shuttle:~/workspace/projects/taped> dbus-monitor --session "type='signal',sender='org.mpris.MediaPlayer2.spotify',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',arg0='org.mpris.MediaPlayer2.Player'"
signal time=1779386357.535682 sender=org.freedesktop.DBus -> destination=:1.127 serial=4294967295 path=/org/freedesktop/DBus; interface=org.freedesktop.DBus; member=NameAcquired
   string ":1.127"
signal time=1779386357.535705 sender=org.freedesktop.DBus -> destination=:1.127 serial=4294967295 path=/org/freedesktop/DBus; interface=org.freedesktop.DBus; member=NameLost
   string ":1.127"
signal time=1779386873.600426 sender=:1.119 -> destination=(null destination) serial=130 path=/org/mpris/MediaPlayer2; interface=org.freedesktop.DBus.Properties; member=PropertiesChanged
   string "org.mpris.MediaPlayer2.Player"
   array [
      dict entry(
         string "Metadata"
         variant             array [
               dict entry(
                  string "mpris:trackid"
                  variant                      string "/com/spotify/track/7Lf7oSEVdzZqTA0kEDSlS5"
               )
               dict entry(
                  string "mpris:length"
                  variant                      uint64 288333000
               )
               dict entry(
                  string "mpris:artUrl"
                  variant                      string "https://i.scdn.co/image/ab67616d0000b273346a5742374ab4cf9ed32dee"
               )
               dict entry(
                  string "xesam:album"
                  variant                      string "Justified"
               )
               dict entry(
                  string "xesam:albumArtist"
                  variant                      array [
                        string "Justin Timberlake"
                     ]
               )
               dict entry(
                  string "xesam:artist"
                  variant                      array [
                        string "Justin Timberlake"
                     ]
               )
               dict entry(
                  string "xesam:autoRating"
                  variant                      double 0.81
               )
               dict entry(
                  string "xesam:discNumber"
                  variant                      int32 1
               )
               dict entry(
                  string "xesam:title"
                  variant                      string "Cry Me a River"
               )
               dict entry(
                  string "xesam:trackNumber"
                  variant                      int32 5
               )
               dict entry(
                  string "xesam:url"
                  variant                      string "https://open.spotify.com/track/7Lf7oSEVdzZqTA0kEDSlS5"
               )
            ]
      )
   ]
   array [
   ]
signal time=1779386891.754381 sender=:1.119 -> destination=(null destination) serial=139 path=/org/mpris/MediaPlayer2; interface=org.freedesktop.DBus.Properties; member=PropertiesChanged
   string "org.mpris.MediaPlayer2.Player"
   array [
      dict entry(
         string "CanGoNext"
         variant             boolean false
      )
      dict entry(
         string "CanGoPrevious"
         variant             boolean false
      )
      dict entry(
         string "CanSeek"
         variant             boolean false
      )
      dict entry(
         string "Metadata"
         variant             array [
               dict entry(
                  string "mpris:trackid"
                  variant                      string "/com/spotify/ad/2b5a5d0c7d4d4b78a84df74a5c94ba8e"
               )
               dict entry(
                  string "mpris:length"
                  variant                      uint64 30000000
               )
               dict entry(
                  string "mpris:artUrl"
                  variant                      string ""
               )
               dict entry(
                  string "xesam:album"
                  variant                      string ""
               )
               dict entry(
                  string "xesam:albumArtist"
                  variant                      array [
                        string ""
                     ]
               )
               dict entry(
                  string "xesam:artist"
                  variant                      array [
                        string ""
                     ]
               )
               dict entry(
                  string "xesam:autoRating"
                  variant                      double 0
               )
               dict entry(
                  string "xesam:discNumber"
                  variant                      int32 0
               )
               dict entry(
                  string "xesam:title"
                  variant                      string "Hör Musik ohne Werbepausen."
               )
               dict entry(
                  string "xesam:trackNumber"
                  variant                      int32 0
               )
               dict entry(
                  string "xesam:url"
                  variant                      string "https://open.spotify.com/ad/2b5a5d0c7d4d4b78a84df74a5c94ba8e"
               )
            ]
      )
   ]
   array [
   ]
signal time=1779386920.918095 sender=:1.119 -> destination=(null destination) serial=145 path=/org/mpris/MediaPlayer2; interface=org.freedesktop.DBus.Properties; member=PropertiesChanged
   string "org.mpris.MediaPlayer2.Player"
   array [
      dict entry(
         string "Volume"
         variant             double 1.00002
      )
   ]
   array [
   ]
signal time=1779386922.685968 sender=:1.119 -> destination=(null destination) serial=146 path=/org/mpris/MediaPlayer2; interface=org.freedesktop.DBus.Properties; member=PropertiesChanged
   string "org.mpris.MediaPlayer2.Player"
   array [
      dict entry(
         string "Volume"
         variant             double 1
      )
      dict entry(
         string "Metadata"
         variant             array [
               dict entry(
                  string "mpris:trackid"
                  variant                      string "/com/spotify/ad/c5f532f3598242249e2d4cec0d540851"
               )
               dict entry(
                  string "mpris:length"
                  variant                      uint64 29000000
               )
               dict entry(
                  string "mpris:artUrl"
                  variant                      string ""
               )
               dict entry(
                  string "xesam:album"
                  variant                      string ""
               )
               dict entry(
                  string "xesam:albumArtist"
                  variant                      array [
                        string ""
                     ]
               )
               dict entry(
                  string "xesam:artist"
                  variant                      array [
                        string ""
                     ]
               )
               dict entry(
                  string "xesam:autoRating"
                  variant                      double 0
               )
               dict entry(
                  string "xesam:discNumber"
                  variant                      int32 0
               )
               dict entry(
                  string "xesam:title"
                  variant                      string "Hör Musik ohne Werbepausen."
               )
               dict entry(
                  string "xesam:trackNumber"
                  variant                      int32 0
               )
               dict entry(
                  string "xesam:url"
                  variant                      string "https://open.spotify.com/ad/c5f532f3598242249e2d4cec0d540851"
               )
            ]
      )
   ]
   array [
   ]
signal time=1779386953.148763 sender=:1.119 -> destination=(null destination) serial=152 path=/org/mpris/MediaPlayer2; interface=org.freedesktop.DBus.Properties; member=PropertiesChanged
   string "org.mpris.MediaPlayer2.Player"
   array [
      dict entry(
         string "Metadata"
         variant             array [
               dict entry(
                  string "mpris:trackid"
                  variant                      string "/com/spotify/ad/0edcaa015d7e4326afafb4acb7e0f35a"
               )
               dict entry(
                  string "mpris:length"
                  variant                      uint64 0
               )
               dict entry(
                  string "mpris:artUrl"
                  variant                      string "https://i.scdn.co/image/ab67616600001e0123d2d1d9f44b73fe14e47068"
               )
               dict entry(
                  string "xesam:album"
                  variant                      string ""
               )
               dict entry(
                  string "xesam:albumArtist"
                  variant                      array [
                        string ""
                     ]
               )
               dict entry(
                  string "xesam:artist"
                  variant                      array [
                        string ""
                     ]
               )
               dict entry(
                  string "xesam:autoRating"
                  variant                      double 0
               )
               dict entry(
                  string "xesam:discNumber"
                  variant                      int32 0
               )
               dict entry(
                  string "xesam:title"
                  variant                      string "—"
               )
               dict entry(
                  string "xesam:trackNumber"
                  variant                      int32 0
               )
               dict entry(
                  string "xesam:url"
                  variant                      string "https://open.spotify.com/ad/0edcaa015d7e4326afafb4acb7e0f35a"
               )
            ]
      )
   ]
   array [
   ]
signal time=1779386960.281891 sender=:1.119 -> destination=(null destination) serial=159 path=/org/mpris/MediaPlayer2; interface=org.freedesktop.DBus.Properties; member=PropertiesChanged
   string "org.mpris.MediaPlayer2.Player"
   array [
      dict entry(
         string "CanGoNext"
         variant             boolean true
      )
      dict entry(
         string "CanGoPrevious"
         variant             boolean true
      )
      dict entry(
         string "CanSeek"
         variant             boolean true
      )
      dict entry(
         string "Metadata"
         variant             array [
               dict entry(
                  string "mpris:trackid"
                  variant                      string "/com/spotify/track/7g2BBjUQWJDdRohU8mOOZk"
               )
               dict entry(
                  string "mpris:length"
                  variant                      uint64 285570000
               )
               dict entry(
                  string "mpris:artUrl"
                  variant                      string "https://i.scdn.co/image/ab67616d0000b2731184191625b12259967b7116"
               )
               dict entry(
                  string "xesam:album"
                  variant                      string "Hypnotica"
               )
               dict entry(
                  string "xesam:albumArtist"
                  variant                      array [
                        string "Benny Benassi"
                     ]
               )
               dict entry(
                  string "xesam:artist"
                  variant                      array [
                        string "Benny Benassi"
                     ]
               )
               dict entry(
                  string "xesam:autoRating"
                  variant                      double 0.02
               )
               dict entry(
                  string "xesam:discNumber"
                  variant                      int32 1
               )
               dict entry(
                  string "xesam:title"
                  variant                      string "Satisfaction - Isak Original Extended"
               )
               dict entry(
                  string "xesam:trackNumber"
                  variant                      int32 1
               )
               dict entry(
                  string "xesam:url"
                  variant                      string "https://open.spotify.com/track/7g2BBjUQWJDdRohU8mOOZk"
               )
            ]
      )
   ]
   array [
   ]
^C
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
