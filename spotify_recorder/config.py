# Copyright (C) 2019-2021 Patrick Ziegler
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

import argparse
import os
import sys

__version__ = "1.1.4"


class ConfigManager:

    def __init__(self):
        parser = argparse.ArgumentParser(
            description="Monitoring Spotify on D-Bus for recording currently played music"
        )

        parser.add_argument("prefix",
                            type=str,
                            nargs="?",
                            default="",
                            help="Output folder (optional)"
                            )

        parser.add_argument("-f", "--format",
                            type=str,
                            dest="format",
                            default="mp3",
                            help="Format for file conversion"
                            )

        parser.add_argument("-b", "--bitrate",
                            type=str,
                            dest="bitrate",
                            default="128k",
                            help="Bitrate for file conversion"
                            )

        parser.add_argument("--version",
                            action='version',
                            version="SpotifyRecorder v" + __version__ +
                            ", Copyright (C) 2019-2021 Patrick Ziegler"
                            )

        args = parser.parse_args()

        self.prefix = args.prefix
        self.format = args.format
        self.bitrate = args.bitrate
