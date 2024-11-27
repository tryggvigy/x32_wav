# Python moudle for Behringer X-Live SD recording card
# This module contains functions for convert to and from the X live WAVe format


from .helpers import *

import struct
import os
from timeit import default_timer as timer


################################################################################################
## name session
# this function gives name to the X-live session to be shown on the X32 UI

# parameters
# name 		(string) max 19 characters

def nameSession(name_str):
	""" Give a name to a session to be displayed on the X32 UI  """
	#open log file
	try:
		log=open("se_log.bin","rb")
	except:
		print "log file not found !"
		return 

	log_copy= log.read()

	log.close()

	log_copy_l=list(log_copy)

	name_p=4*(388)

	name_len= len(name_str)

	for i in range(19):
		if i < name_len:
			log_copy_l[name_p]=name_str[i]
		else:
			log_copy_l[name_p]="\x00"

		name_p+=1

	#char 20 is ZERO
	log_copy_l[name_p]="\x00"

	log=open("se_log.bin","wb")

	log.write("".join(log_copy_l))
	log.close()

	print "Session named successfully!"
	


################################################################################################
## Get session info
# this function returns the information about a session

# parameters

def getSessionInfo():
	""" Get information about session """
	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 

	total_length_bytes=total_length*4

	print "no_channels =" + str(no_channels)
	print "sample_rate =" + str(sample_rate)
	print "no_takes =" + str(no_takes)
	print "no_markers =" + str(no_markers)
	print "Total audio length in bytes = " + str(total_length_bytes)
	print "Total audio samples per channel = " + str(total_length)
	print "Total audio time per channel = " + str(total_length/sample_rate)

	for i in range(no_markers):
		print "Marker " + str(i) + " at " + str(take_markers[i]) +" samples or " + str(take_markers[i]/sample_rate) + " seconds"




################################################################################################	
## Extract session
# this function extract all channels to single WAV files and the complete length of the session

# NOTE 1: maximal session length is 6,21 hrs at 48KHz
# NOTE 1: maximal session length is 6,76 hrs at 44,1KHz

# parameters

def extractSession():
	""" Extract all channels from the complete session length"""
	
	start=timer()

	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 

	take=[] 

	buf_size= 1024*1024*4				#Bytes  DO NOTE CHANGE!!


	try:
		os.mkdir( "sesssion_"+ session_str, 0755)
	except:
		print "please remove existing folder"
		return

	waves=create_waves("sesssion_"+ session_str, total_length,sample_rate, no_channels)
	print "Unpacking audio data, this may take a  while :) \n" 

	#get data of takes and write files

	for i in range(no_takes):
		#open take
		if openTake(i,take,take_size) == 1:
			return

		#jump wave header
		take[i].seek(32*1024,0)

		readWriteAudio(take[i],take_size[i],buf_size,no_channels,waves)

	
	#close last audio files 
	close_waves(waves,no_channels)
	# close takes
	for i in range(no_takes):
		take[i].close()


	end=timer()


	print "process completed in="+str(end-start)+"sec"
	#raw_input()


## Extract a single channel
# this function extract a single channels to single WAV files and the complete length of the session

# NOTE 1: maximal session length is 6,21 hrs at 48KHz
# NOTE 1: maximal session length is 6,76 hrs at 44,1KHz

# parameters
# channel number		(unsigned integer)    channel to be extract
def extractChannel(channel_no):
	""" Extract a single channel from the complete session length"""
	start=timer()

	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 

	take=[] 

	buf_size= 1024*1024*4				#Bytes  DO NOTE CHANGE!!


	try:
		os.mkdir( "channel_" +str(channel_no) + "_"+ session_str, 0755)
	except:
		print "please remove existing folder"
		return

	wave=create_wave("channel_" +str(channel_no) + "_" + session_str, total_length,sample_rate, channel_no)
	print "Unpacking audio data, this may take a  while :) \n" 

	#get data of takes and write files
	for i in range(no_takes):
		#open take
		if openTake(i,take,take_size) == 1:
			return

		#jump wave header
		take[i].seek(32*1024,0)  

		readWriteAudio_Ch(take[i],take_size[i],buf_size,no_channels,wave,channel_no)

	
	#close last audio files 
	wave[0].close()

	for i in range(no_takes):
		take[i].close()

	end=timer()

	print "process completed in="+str(end-start)+"sec"
	#raw_input()


## Extract session markers
# this function extract all channels to single WAV files from marker to marker

# NOTE 1: maximal session length is 6,21 hrs at 48KHz
# NOTE 1: maximal session length is 6,76 hrs at 44,1KHz

# parameters
# start_marker			(unsigned integer) time point to begin in seconds
# stop_marker			(unsigned integer) marker number to stop
def extractSessionMarker(start_marker, stop_marker):
	""" Extract all channels, form marker to maker """

	start=timer()

	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 

	take=[] 

	buf_size= 1024*1024*4				#Bytes  DO NOTE CHANGE!!

	folder_name = "sesssion_marker_%d_%d_" %(start_marker,stop_marker) + session_str

	try:
		os.mkdir( folder_name, 0755)
	except:
		print "please remove existing folder"
		return

	waves=create_waves(folder_name, total_length,sample_rate, no_channels)
	print "Unpacking audio data, this may take a  while :) \n" 

	(start_take,end_take,s_time_x_ch,e_time_x_ch)=calcLimitsMarker(start_marker,stop_marker,no_takes,take_size,take_markers,no_channels)

	#get data of takes and write files
	for i in range(start_take,(end_take+1)):
		#open take
		if openTake(i,take,take_size) == 1:
			return

		#jump wave header
		take[i].seek(32*1024,0)

		l_takesize=calcTakeLen(i,start_take,take,take_size,end_take,s_time_x_ch,e_time_x_ch)

		readWriteAudio(take[i],l_takesize,buf_size,no_channels,waves)



	#close last audio files 
	close_waves(waves,no_channels)
	# close takes
	for i in range(start_take,(end_take+1)):
		take[i].close()


	end=timer()


	print "process completed in="+str(end-start)+"sec"


## Extract a single channel
# this function extract a single channels to single WAV files from marker to marker

# NOTE 1: maximal session length is 6,21 hrs at 48KHz
# NOTE 1: maximal session length is 6,76 hrs at 44,1KHz

# parameters
# channel number		(unsigned integer)    channel to be extract
# start_marker			(unsigned integer) marker number to begin
# stop_marker			(unsigned integer) marker number to stop
def extractChannelMarker( channel_no, start_marker, stop_marker):
	""" Extract a single, form marker to maker"""

	start=timer()

	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 

	take=[] 

	buf_size= 1024*1024*4				#Bytes  DO NOTE CHANGE!!

	folder_name = "channel_marker_%d_%d_" %(start_marker,stop_marker) + str(channel_no) + "_"+ session_str

	try:
		os.mkdir( folder_name, 0755)
	except:
		print "please remove existing folder"
		return

	wave=create_wave(folder_name, total_length,sample_rate, channel_no)
	print "Unpacking audio data, this may take a  while :) \n" 


	(start_take,end_take,s_time_x_ch,e_time_x_ch)=calcLimitsMarker(start_marker,stop_marker,no_takes,take_size,take_markers,no_channels)

	#get data of takes and write files
	for i in range(start_take,(end_take+1)):
		#open take
		if openTake(i,take,take_size) == 1:
			return
		#jump wave header
		take[i].seek(32*1024,0)  

		l_takesize=calcTakeLen(i,start_take,take,take_size,end_take,s_time_x_ch,e_time_x_ch)

		readWriteAudio_Ch(take[i],l_takesize,buf_size,no_channels,wave,channel_no)


	
	#close last audio files 
	wave[0].close()

	for i in range(start_take,(end_take+1)):
		take[i].close()

	end=timer()

	print "process completed in="+str(end-start)+"sec"


## Extract session user defined
# this function extract all channels to single WAV files from a time point to a time point

# NOTE 1: maximal session length is 6,21 hrs at 48KHz
# NOTE 1: maximal session length is 6,76 hrs at 44,1KHz

# parameters
# start_time			(unsigned integer) time point to begin in seconds
# stop_time				(unsigned integer) time point to stop in seconds
def extractSessionTime( start_time, stop_time):
	""" Extract all channels, a time point to a time point """

	start=timer()

	start_time = int(start_time)
	stop_time = int(stop_time)

	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 

	take=[] 

	buf_size= 1024*1024*4				#Bytes  DO NOTE CHANGE!!

	folder_name = "sesssion_time_%d_%d_" %(start_time,stop_time) + session_str

	try:
		os.mkdir( folder_name, 0755)
	except:
		print "please remove existing folder"
		return

	waves=create_waves(folder_name, total_length,sample_rate, no_channels)
	print "Unpacking audio data, this may take a  while :) \n" 

	(start_take,end_take,s_time_x_ch,e_time_x_ch)=calcLimitsTime(start_time,stop_time,no_takes,take_size,sample_rate,no_channels)


	#get data of takes and write files
	for i in range(start_take,(end_take+1)):
		#open take
		if openTake(i,take,take_size) == 1:
			return

		#jump wave header
		take[i].seek(32*1024,0)

		l_takesize=calcTakeLen(i,start_take,take,take_size,end_take,s_time_x_ch,e_time_x_ch)

		readWriteAudio(take[i],l_takesize,buf_size,no_channels,waves)


	#close last audio files 
	close_waves(waves,no_channels)
	# close takes
	for i in range(start_take,(end_take+1)):
		take[i].close()


	end=timer()


	print "process completed in="+str(end-start)+"sec"


## Extract a single channel
# this function extract a single channels to single WAV files a time point to a time point

# NOTE 1: maximal session length is 6,21 hrs at 48KHz
# NOTE 1: maximal session length is 6,76 hrs at 44,1KHz

# parameters
# channel number		(unsigned integer)  channel to be extract
# start_time			(unsigned integer) 	time point to begin in seconds
# stop_time				(unsigned integer)  time point to stop in seconds
def extractChannelTime(channel_no, start_time, stop_time):
	""" Extract a single, a time point to a time point"""
	start=timer()

	start_time = int(start_time)
	stop_time = int(stop_time)

	(session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)=readLogFile()
	if no_channels==0 or sample_rate==0:
		return 
		
	take=[] 

	buf_size= 1024*1024*4				#Bytes  DO NOTE CHANGE!!

	folder_name = "channel_time_%d_%d_" %(start_time,start_time) + str(channel_no) + "_"+ session_str

	try:
		os.mkdir( folder_name, 0755)
	except:
		print "please remove existing folder"
		return

	wave=create_wave(folder_name, total_length,sample_rate, channel_no)
	print "Unpacking audio data, this may take a  while :) \n" 


	(start_take,end_take,s_time_x_ch,e_time_x_ch)=calcLimitsTime(start_time,stop_time,no_takes,take_size,sample_rate,no_channels)

	#get data of takes and write files
	for i in range(start_take,(end_take+1)):
		#open take
		if openTake(i,take,take_size) == 1:
			return

		#jump wave header
		take[i].seek(32*1024,0)  

		l_takesize=calcTakeLen(i,start_take,take,take_size,end_take,s_time_x_ch,e_time_x_ch)
#
		readWriteAudio_Ch(take[i],l_takesize,buf_size,no_channels,wave,channel_no)

	
	#close last audio files 
	wave[0].close()

	for i in range(start_take,(end_take+1)):
		take[i].close()

	end=timer()



	print "process completed in="+str(end-start)+"sec"