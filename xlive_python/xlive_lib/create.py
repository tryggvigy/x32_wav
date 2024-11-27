# Python moudle for Behringer X-Live SD recording card
# This module contains functions for convert to and from the X live WAVe format

from .helpers import *

import struct
import os
import math
import time
from timeit import default_timer as timer

################################################################################################
## creat X-Live session
# this function creates a session folder that can be read by the X-LIVE ext-card in the X32
# the wave channels are read automatically and need to be name as follow:
# ch1.wav to ch32.wav
#
# Format must be:
# 24 bit PCM
# 48000/44100 Hz sample rate
# max 32 channels
# all channles most be of the same length

# parameters
# name_str		(string) path to folder containing session max 19 characters
# markers		(float list) list of desired markers, values must be in seconds; [] means no markers

def createSession(name_str, markers):
	""" Create an X-live session"""
	start=timer()

	max_take_size = (4*1024*1024*1024) - (32*1024*2)		#4GB - 32KB - 32KB header

	waves= []
	#open wave files
	for i in range(32):
		try:
			waves.append(open("ch_%d.wav"%(i+1),"rb"))
		except:
			no_chls = i
			break

	if no_chls==0:
		print "no wave files found \n"
		return

	print "%d wav files found \n" %(no_chls)

	# calc fill channes, in case the number of wav files does not match 8, 16 or 32
	if no_chls <= 8:
		fill_chls = 8 - no_chls
	elif no_chls <= 16:
		fill_chls = 16 - no_chls
	elif no_chls <= 32:
		fill_chls = 32 - no_chls
	else:
		no_chls = 32							#limit to 32 channles
		print "more than 32 channels found, exceeding channels will be ignored! \n "

	#read wav header
	file_size=[]
	data_size=[]
	wav_samp_rate=[]

	for i in range(no_chls):
		riff=waves[i].read(4)
		if riff != "RIFF":
			print "ch_%d.wav not a RIFF file! \n" %(i+1)
			return

		file_size.append(int(waves[i].read(4)[::-1].encode('hex'),16) )# + 8
		wave=waves[i].read(4)
		if wave != "WAVE":
			print "ch_%d.wav not a WAVE file! \n" %(i+1)
			return

		leave = 0
		while leave != 2:
			temp=waves[i].read(4)

			if temp == "fmt ":
				fmt_size = int(waves[i].read(4)[::-1].encode('hex'),16)
				if fmt_size != 16:
					print "ch_%d.wav wrong fmt_size! \n" %(i+1)
					return

				wav_format =int(waves[i].read(2)[::-1].encode('hex'),16)
				if wav_format != 1:
					print "ch_%d.wav WAV format not supported! \n" %(i+1)
					return

				wav_chs =int(waves[i].read(2)[::-1].encode('hex'),16)
				if wav_chs != 1:
					print "ch_%d.wav WAV format not supported! \n" %(i+1)
					return

				wav_samp_rate.append(int(waves[i].read(4)[::-1].encode('hex'),16))
				if wav_samp_rate[i] != 48000 and  wav_samp_rate != 44100:
					print "ch_%d.wav WAV sample rate not supported! \n" %(i+1)
					return

				dwAvgBytesPerSec = int(waves[i].read(4)[::-1].encode('hex'),16)
				wBlockAlign = int(waves[i].read(2)[::-1].encode('hex'),16)

				bits_per_samp = int(waves[i].read(2)[::-1].encode('hex'),16)
				if bits_per_samp != 24:
					print "ch_%d.wav WAV bit resolution not supported! \n" %(i+1)
					return

				leave += 1

			elif temp == "JUNK":
				junk_size = int(waves[i].read(4)[::-1].encode('hex'),16)
				waves[i].seek(junk_size,1)

			elif temp == "data":
				data_size.append(int(waves[i].read(4)[::-1].encode('hex'),16) )

				leave += 1
			else:
				x_size = int(waves[i].read(4)[::-1].encode('hex'),16)
				waves[i].seek(x_size,1)

	# check that all files are of the same length
	audio_len = data_size[0]
	audio_samprate = wav_samp_rate[0]
	for i in range(no_chls):
		if data_size[i] != audio_len:
			print "files are not of the same length! \n"
			return

		if wav_samp_rate[i] != audio_samprate:
			print "files are not of the same sample rate! \n"
			return
 		

	## create xlive session parameters
	datetime = time.gmtime()									# create datetime tag
	
	session_name = ((datetime.tm_year - 1980) << 25) + (datetime.tm_mon << 21) + (datetime.tm_mday << 16 ) + (datetime.tm_hour << 11 ) + (datetime.tm_min << 5 ) + (datetime.tm_sec/2) 
	no_channels = no_chls+fill_chls
	sample_rate = audio_samprate
	date_code = session_name
	no_takes = 0
	no_markers = len(markers)
	total_length = audio_len /3     #samples

	take_size = [] 
	marker_vec = []

	audio_bytes = total_length*4*no_channels					 ## 32 bit by total channels
	#trim it to 32kB bunday
	if audio_bytes % (32*1024):
		audio_bytes -= audio_bytes % (32*1024)

	while (audio_bytes >= max_take_size):
		no_takes += 1
		take_size.append(max_take_size/4)						# in samples
		audio_bytes -= max_take_size


	#the rest
	if audio_bytes != 0:
		no_takes += 1
		take_size.append(audio_bytes/4)							# in samples


	#get markers
	for i in range(no_markers):
		marker_vec.append(int(markers[i]*sample_rate))


	marker_vec.append(0)										# maker_no+1 must be zero

	print "Creating log file... \n"
	dirname= hex(session_name)[2:10] 

	os.mkdir(dirname, 0755)

	#create log file
	log_file=open(dirname + "/se_log.bin","wb")

	log_file.write(struct.pack('<I',session_name))					# session_name; 
	log_file.write(struct.pack('<I',no_channels))					# no_channel;
	log_file.write(struct.pack('<I',sample_rate))					# sample_rate;
	log_file.write(struct.pack('<I',date_code))						# date_code;
	log_file.write(struct.pack('<I',no_takes))						# no_takes;
	log_file.write(struct.pack('<I',no_markers))					# no_markers;
	log_file.write(struct.pack('<I',total_length))					# total_len;                   //in samples; total duration in time
	
	for i in range(no_takes):	
		log_file.write(struct.pack('<I',take_size[i]))				# take_size[MAX_NO_TAKE];      //in samples; channel size is */no_ch  
	for i in range(256 - no_takes):
		log_file.write(struct.pack('<I',0))							# take_size[MAX_NO_TAKE];      //in samples; channel size is */no_ch  

	for i in range(no_markers):
		log_file.write(struct.pack('<I',marker_vec[i]))				# markers[MAX_NO_MAR+1];         //in samples, ref to total duration in time  
	for i in range(101-no_markers):
		log_file.write(struct.pack('<I',0))							# markers[MAX_NO_MAR+1];         //in samples, ref to total duration in time  
	
	for i in range(24):
		log_file.write(struct.pack('<I',0))

	if len(name_str) < 20:
		log_file.write(name_str)									# char name_str[SE_NAME_LEN];    //20/4 = 5   session name string              (20 bytes)  zero terminated!!!
	else:
		log_file.write(name_str[0:19])

	while(log_file.tell()<2048):
		log_file.write(struct.pack('<B',0))

	log_file.close()

	print "Packing audio data, this may take a  while :) \n"


	#for j in range(no_chls):
	#	waves[j].seek(44,0)

	## create take waves
	for i in range(no_takes):
		#create wave
		if (i+1) < 10:
			wave=open(dirname + "/0000000"+ str(i+1)+".wav" ,"wb")		
		elif (i+1) < 100:
			wave=open(dirname + "/000000" + str(i+1)+".wav" ,"wb")	
		else:
			wave=open(dirname + "/00000"  + str(i+1)+".wav" ,"wb")	

		#write wave header
		# header size without junk data is 52, 460B would be need to fill a 32KB cluster 
		junk_bytes = (1024*32) - 52
		wave.write("RIFF")
		wave.write(struct.pack('<I',audio_bytes+44+junk_bytes))  #+8
		wave.write("WAVE")
		wave.write("fmt ")
		wave.write(struct.pack('<I',16))
		wave.write(struct.pack('<H',1))							#wFormatTag
		wave.write(struct.pack('<H',no_channels))
		wave.write(struct.pack('<I',sample_rate))
		wave.write(struct.pack('<I',sample_rate*no_channels*4))	#dwAvgBytesPerSec
		wave.write(struct.pack('<H',no_channels*4))						#wBlockAlign
		wave.write(struct.pack('<H',32))						#wBitsPerSample
		wave.write("JUNK")
		wave.write(struct.pack('<I',junk_bytes)) 	
		for j in range(460):
			wave.write(' ')
		wave.write("data")							#504
		wave.write(struct.pack('<I',audio_bytes)) 	#512

		for j in range(junk_bytes-468):
			wave.write(' ')
		wave.write("data")
		wave.write(struct.pack('<I',audio_bytes)) 

		print take_size[i]
		for k in range(take_size[i]/no_channels):
			#write audio data
			samples=""
			for j in range(no_chls):
				samples+= "\x00"
				samples+= waves[j].read(3)
			wave.write(samples)
				
			#write audio fill data
			for j in range(fill_chls):
				wave.write(struct.pack('<I',0))

		wave.close()
		del(wave)

	end=timer()

	print "process completed in="+str(end-start)+"sec"