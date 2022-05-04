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