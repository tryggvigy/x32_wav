# Python moudle for Behringer X-Live SD recording card
# This module contains functions for convert to and from the X live WAVe format
import struct
import os


##
def readLogFile():

	session_str=""
	session_no=0
	no_channels=0
	sample_rate=0
	date_code=0
	no_takes=0
	no_markers=0
	total_length=0

	take_size= []
	take_markers= []


	#open log file
	try:
		log=open("se_log.bin","rb")
	except:
		print "log file not found !"
		return (0,0,0,0,0,0,0,0,0,0)



	#get paramters
	session_str = log.read(4)[::-1].encode('hex')
	session_no	= int(session_str,16)
	no_channels	= int(log.read(4)[::-1].encode('hex'),16)
	sample_rate	= int(log.read(4)[::-1].encode('hex'),16)
	date_code	= int(log.read(4)[::-1].encode('hex'),16)
	no_takes	= int(log.read(4)[::-1].encode('hex'),16)
	no_markers	= int(log.read(4)[::-1].encode('hex'),16)
	total_length= int(log.read(4)[::-1].encode('hex'),16)		#samples per channel

	##get data of log
	take_size= []
	for i in range(no_takes):
		take_size.append(int(log.read(4)[::-1].encode('hex'),16))				#samples

	#dummy reads  read the rest of the not used take lengths
	for i in range(256-no_takes):			#MAX_NO_TAKE
		log.read(4)

	take_markers=[]
	for i in range(no_markers):
		take_markers.append(int(log.read(4)[::-1].encode('hex'),16))

	#log.seek(0,2)
	#print "file size=" + str(log.tell())

	log.close()

	return (session_str,session_no,no_channels,sample_rate,date_code,no_takes,no_markers,total_length,take_size,take_markers)

##
def create_waves(folder,no_samples,sample_rate, no_waves):
	"""create audio files and write WAVE header"""
	bytes_datachunk=no_samples*3

	chan = []
	for i in range(no_waves):
		chan.append(open( folder+ "/" + "ch_"+str(i+1)+".wav","wb"))
		chan[i].write("RIFF")
		chan[i].write(struct.pack('<I',bytes_datachunk+36))
		chan[i].write("WAVE")
		chan[i].write("fmt ")
		chan[i].write(struct.pack('<I',16))
		chan[i].write(struct.pack('<H',1))						#wFormatTag
		chan[i].write(struct.pack('<H',1))
		chan[i].write(struct.pack('<I',sample_rate))
		chan[i].write(struct.pack('<I',sample_rate*3))			#dwAvgBytesPerSec
		chan[i].write(struct.pack('<H',3))						#wBlockAlign
		chan[i].write(struct.pack('<H',24))							#wBitsPerSample
		chan[i].write("data")
		chan[i].write(struct.pack('<I',bytes_datachunk))

	return chan

##
def close_waves(waves, no_waves):
	for i in range(no_waves):
		waves[i].close()

##
def create_wave(folder,no_samples,sample_rate, ch_number):
	"""create audio files and write WAVE header"""
	bytes_datachunk=no_samples*3

	chan = []
	for i in range(1):
		chan.append(open( folder+ "/" + "ch_"+str(ch_number)+".wav","wb"))
		chan[i].write("RIFF")
		chan[i].write(struct.pack('<I',bytes_datachunk+36))
		chan[i].write("WAVE")
		chan[i].write("fmt ")
		chan[i].write(struct.pack('<I',16))
		chan[i].write(struct.pack('<H',1))									#wFormatTag
		chan[i].write(struct.pack('<H',1))
		chan[i].write(struct.pack('<I',sample_rate))
		chan[i].write(struct.pack('<I',sample_rate*3))				#dwAvgBytesPerSec
		chan[i].write(struct.pack('<H',3))							#wBlockAlign
		chan[i].write(struct.pack('<H',24))							#wBitsPerSample
		chan[i].write("data")
		chan[i].write(struct.pack('<I',bytes_datachunk))

	return chan


##
def readWriteAudio(take,takesize,bufsize,no_channels,waves_to):
	#read and write audio
	for k in range((takesize*4)/bufsize):
		read_buf=take.read(bufsize)


		for j in range(no_channels):
			idx_read_buf=j*4
			ch_buffer= ""
			for k in range(bufsize/(no_channels*4)):
				ch_buffer+=read_buf[(idx_read_buf+1):(idx_read_buf+4)]  # 3 bytes
				idx_read_buf=idx_read_buf+ no_channels*4

			waves_to[j].write((ch_buffer))


	# rest not multple of buffer
	buf_size_rest= (takesize*4) % bufsize
	read_buf=take.read(buf_size_rest)
	idx_read_buf=0


	for j in range(no_channels):
		idx_read_buf=j*4
		ch_buffer= ""
		for k in range(buf_size_rest/(no_channels*4)):
			ch_buffer+=read_buf[(idx_read_buf+1):(idx_read_buf+4)]  # 3 bytes
			idx_read_buf=idx_read_buf+ no_channels*4

		waves_to[j].write((ch_buffer))

##
def readWriteAudio_Ch(take,takesize,bufsize,no_channels,wave_to,channel_no):
	#read and write audio
	for k in range((takesize*4)/bufsize):
		read_buf=take.read(bufsize)


		for j in range(no_channels):
			idx_read_buf=j*4
			if j == (channel_no-1):
				ch_buffer= ""
				for k in range(bufsize/(no_channels*4)):
					ch_buffer+=read_buf[(idx_read_buf+1):(idx_read_buf+4)]  # 3 bytes
					idx_read_buf=idx_read_buf+ no_channels*4

				wave_to[0].write((ch_buffer))


	# rest not multple of buffer
	buf_size_rest= (takesize*4) % bufsize
	read_buf=take.read(buf_size_rest)
	idx_read_buf=0


	for j in range(no_channels):
		idx_read_buf=j*4
		if j == (channel_no-1):
			ch_buffer= ""
			for k in range(buf_size_rest/(no_channels*4)):
				ch_buffer+=read_buf[(idx_read_buf+1):(idx_read_buf+4)]  # 3 bytes
				idx_read_buf=idx_read_buf+ no_channels*4

			wave_to[0].write((ch_buffer))

##
def calcTakeLen(i,start_take,take,take_size,end_take,s_time_x_ch,e_time_x_ch):
	#calcualte take length
	if i == start_take:
		take[i].seek(s_time_x_ch*4,1)					#jump to start point
		if i == end_take:
			l_takesize = e_time_x_ch - s_time_x_ch
		else:
			l_takesize = take_size[i] - s_time_x_ch

	elif i == end_take:
		l_takesize = e_time_x_ch
	else:
		l_takesize = take_size[i]

	return l_takesize

##
def openTake(i,take,take_size):
	#open take
	try:
		if (i+1) < 10:
			take.append(open("0000000"+ str(i+1)+".wav" ,"rb"))
		elif (i+1) < 100:
			take.append(open("000000" + str(i+1)+".wav" ,"rb"))
		else:
			take.append(open("00000" + str(i+1)+".wav" ,"rb"))
	except:
		print "take no %d not found! \n" % (i)
		return 1

	print "reading take %d \n" % (i+1)
	print "take length %d \n" % (take_size[i])

	return 0
##
def calcLimitsTime(start_time,stop_time,no_takes,take_size,sample_rate,no_channels):
	#find out start take
	start_take=0
	s_time_x_ch = start_time* sample_rate * no_channels
	time_compare=0
	for i in range(no_takes):
		time_compare += take_size[i]
		if s_time_x_ch < time_compare:
			start_take = i
			break

	#find out stop take
	end_take=0
	e_time_x_ch = stop_time * sample_rate * no_channels
	time_compare=0
	for i in range(no_takes):
		time_compare += take_size[i]
		if e_time_x_ch < time_compare:
			end_take = i
			break

	return (start_take,end_take,s_time_x_ch,e_time_x_ch)

##
def calcLimitsMarker(start_marker,stop_marker,no_takes,take_size,take_markers,no_channels):
	#find out start take
	start_take=0
	s_time_x_ch = take_markers[start_marker] * no_channels
	time_compare=0
	for i in range(no_takes):
		time_compare += take_size[i]
		if s_time_x_ch < time_compare:
			start_take = i
			break

	#find out stop take
	end_take=0
	e_time_x_ch = take_markers[stop_marker]  * no_channels
	time_compare=0
	for i in range(no_takes):
		time_compare += take_size[i]
		if e_time_x_ch < time_compare:
			end_take = i
			break

	return (start_take,end_take,s_time_x_ch,e_time_x_ch)


########################################################################
########################################################################
########################################################################


def create_header(folder,no_samples):
#create audio files and write WAVE header
	size=no_samples*4*no_channels

	chan.append(open( folder+ "/" + "sesssion"+session_no_str+".wav","wb"))
	chan[0].write("RIFF")
	chan[0].write(struct.pack('<I',(size)+44+460))
	chan[0].write("WAVE")
	chan[0].write("fmt ")
	chan[0].write(struct.pack('<I',16))
	chan[0].write(struct.pack('<H',1))							#wFormatTag
	chan[0].write(struct.pack('<H',no_channels))
	chan[0].write(struct.pack('<I',sample_rate))
	chan[0].write(struct.pack('<I',sample_rate*no_channels*4))	#dwAvgBytesPerSec
	chan[0].write(struct.pack('<H',no_channels*4))						#wBlockAlign
	chan[0].write(struct.pack('<H',32))						#wBitsPerSample
	chan[0].write("JUNK")
	chan[0].write(struct.pack('<I',460))

	for k in range(460):
		chan[0].write(' ')

	chan[0].write("data")
	chan[0].write(struct.pack('<I',size))
# header size without junk data is 52, 460B would be need to comple at 512B sector
# the max file size shuold be 2GB/4GB - expand size