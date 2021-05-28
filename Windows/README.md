# Android Microphone (Windows side)  

Android Microphone Project (Windows Application folder)  

------

### Structure

Four major threads:  
* UI thread  
* Bluetooth server thread  
* USB tcp server thread
* Audio recorder thread  

------

#### UI Thread  

* manage the rest 2 threads  
* handle button clicks  
* display messages  

#### Bluetooth Thread  

* start bluetooth server  
* validate connected client  
* establish connection  
* receive audio data  
* cancel connection  
* stop server  

#### USB Thread  

* start USB tcp server  
* select server address  
* establish connection
* receive audio data  
* cancel connection  
* stop server  

#### Audio Thread  

* start wave out player  
* add recevied data to samples  
* stop playing  
* update audio device  

------

### Notes

I use `32feet.net` for bluetooth functions and `NAudio` for playing raw audio data. Though it is not the first WPF application I wrote, it is the first one within a year, so I need to search a lot.  

For displaying audio wave, I learned a lot from online resources. Right now the fastest way I discovered is to store a fixed-sized list of `Point`, and update the positions of those points each time. Then I create a `PointCollection` object and add all `Point` data from the list. At last, update the `Polygon` which is also stored, and update the `Children` of the canvas object. This way, I'm avoiding most garbage collection actions.  

My method of displaying raw byte audio array in real time is:  
1. First set a screen size (I choose 2048, which means 2048 `short`s, or 4096 bytes)  
2. Whenever new data is received from stream, run a while loop and check the maximum float (at least 0) and minimum float (at most 0) in current screen  
3. If current screen is full, add current max and min `short` values to the wave display  

Another interesting thing is that since I already converted the `short` array to `byte` array based on Big Endian on Android side, I don't need to reverse bytes to get a `short` value from the stream.  