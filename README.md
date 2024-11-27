#
Rust and Cargo must be installed on the computer!!

# General steps for extracting audio of an X-LIVE session
1.- Copy Session folder (e.g. 4ADC5F3B) from SD card into the computer
2.- Copy the "xlive_lib" folder into the session folder"
3.- Open a command prompt and navigate to the session folder
4.- Run the Rust CLI tool by typing
	> cargo run -- <command> [arguments]

#name a session max 19 characters
5.- type:
	> cargo run -- nameSession "hello world"

#get session info
5.- type:
	> cargo run -- getSessionInfo

#Extract all audio channels of an X-LIVE session
5.- type:
	> cargo run -- extractSession

#Extract all audio channels of an X-LIVE session of user defined time span from 0 sec to 120sec
5.- type:
	> cargo run -- extractSessionTime 0 120

#Extract all audio channels of an X-LIVE session from marker x to marker y
5.- get session info to find out the markers list and their index:
	> cargo run -- getSessionInfo
	> cargo run -- extractSessionMarker 1 2

#Extract a single channel of an X-LIVE session, e.g. channel 3
5.- type:
	> cargo run -- extractChannel 3

#Extract a single channel of an X-LIVE session of user defined time span from 0 sec to 120sec, e.g. channel 3
5.- type:
	> cargo run -- extractChannelTime 3 0 120

#Extract a single channel of an X-LIVE session from marker x to marker y, e.g. channel 3
5.- get session info to find out the markers list and their index:
	> cargo run -- getSessionInfo
	> cargo run -- extractChannelMarker 3 1 2

####

# create an X-live session out of single channels
#Requirements
- Audio files must be uncompressed WAV file
	- one audio channel per file
	- all the same sample rate:  48000 or 441000
	- all must be 24 bit PCM coded

- Files must be named ch_1 to ch_32
- The number of audio files can be from 1 to 32, the function would calculate the required channels and fill the none used ones

# steps
1.- Create a folder and copy the audio files in it
2.- Copy the "xlive_lib" folder inside the folder containing the audio files
3.- Open a command prompt and navigate to the session folder
4.- Run the Rust CLI tool by typing
	> cargo run -- <command> [arguments]

5.- 2 parameters are needed, a string of max 19 characters as a session name, and a list of markers
	for no markers type:
	> cargo run -- createSession "Hello World" []

	for markers, markers are given in seconds
	> cargo run -- createSession "Hello World" [10.5,120.8,130]