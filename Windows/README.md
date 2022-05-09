# Android Microphone (Windows side)  

AndroidMic Project (Windows Application folder)  
Built with WPF  

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

### Latency  

By test, audio via bluetooth socket has much __higher latency__ than TCP socket through Wifi on my machine, even though the code is very similar.  
VB Cable can be configured to minimum latency, but still slower than physical devices.  
Android `AudioRecord` also has latency for recording. Can look into [Oboe](https://github.com/google/oboe) to replace `AudioRecord` to improve.  


### SpeexDSP and Audio Processing  

This projects integrates [SpeexDSP](https://gitlab.xiph.org/xiph/speexdsp) library to support echo cancellation, noise suppression, automatic gain control and voice activity detection. The library dll provided in this project is compiled locally from the [latest source code](https://gitlab.xiph.org/xiph/speexdsp/-/commit/68311d46785be76d2a186c75578d51108bff6dfb) for x86. Then I wrote a C# binding for the library to call the functions in header files.  
It is tricky to configure SpeexDSP filter, especially for echo cancellation. I looked at a lot of open source projects to learn their parameters. I tweaked and tested those parameters to better fit my app. For echo cancellation, a player buffer is required, which should be the samples sent to the speaker. I use NAudio provider which calls WASAPI internally to record the PC soundcard output.  
Alternative for audio processing is [WebRTC](https://webrtc.org/). It is huge library in comparison, so I didn't include it in the end.  