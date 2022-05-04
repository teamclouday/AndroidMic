# Android Microphone (Windows side)  

AndroidMic Project (Windows Application folder)  

------

## Structure

### UI Thread  

* handle button clicks  
* handle callback events  
* display messages  

### Stream Manager Thread  

* start bluetooth/wifi server  
* validate connected client  
* establish connection  
* receive audio data  
* cancel connection  
* stop server  

### Audio Manager Thread  

* open player  
* add recevied data to samples  
* stop playing  
* choose audio device  
* set volume  

------

## Some Notes

### TCP vs UDP

I once thought TCP delays audio data transfer. So I went to look at RTSP as alternative. Turns out it uses TCP for control and UDP/TCP to transfer data. RTSP is mainly used to stream video (with audio) data. TCP is still fast enough for audio data. No stream control required in my app, so RTSP is not necessary. BTW, UDP transfer is not easy to implement without a sequential order manager.

### Buffers

A lot of buffers used in one audio pass:  
1. Android `AudioRecord` will first store audio data in a buffer  
2. Then recorded data will be copied to `AudioBuffer` by audio manager, which will be read by stream manager  
3. Stream manager reads and sends to Windows side. Windows stream manager receives and stores in `AudioBuffer`, which will be read by audio manager  
4. Audio manager has a `BufferedWaveProvider` layer for `NAudio` player, which will also store cached audio data  

The size and implementation of these buffers affect the latency of audio transfer. Two `AudioBuffer`s can be configured. I set max number of buffers to `3`. (Can also try 2 but it may drop audio too frequently) For `NAudio` wave out player, I set desired latency to `50`, number of buffers to `3`. A combination of (50,2) will cause choppy audio.

Assume Android `AudioRecord` has optimum performance. Audio format is `16000` sample rate, `16` bits (2 bytes) data, `1` channel, PCM. Expect to have 16000x2x1=`32000` bytes generated from Android app per second. Bluetooth is able to transfer at least 1Mbps. Wifi will be much faster (up to 2 Gbps). TCP header can be up to 60 bytes per packet. So sockets are not expected to cause delay.