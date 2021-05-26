# Android Microphone (Windows side)  

Android Microphone Project (Windows Application folder)  

------

### Structure

Three major threads:  
* UI thread  
* Bluetooth client thread  
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

#### Audio Thread  

* start wave out player  
* add recevied data to samples  
* stop playing  
* update audio device  

------

### Notes

I use `32feet.net` for bluetooth functions and `NAudio` for playing raw audio data. Though it is not the first WPF application I wrote, it is the first one within a year, so I need to search a lot.  
