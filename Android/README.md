# Android Microphone (Android side)  

AndroidMic Project (Android Application folder)

------

## Structure

### _Activity_
#### UI Thread  

* handle button and switch clicks  
* display messages  

#### Message Handler Thread

* communicate with service

### _Service_
#### Stream Manager Thread  

* check for permission  
* search for valid PC  
* validate connected PC  
* establish connection  
* transfer stored audio data  
* cancel connection  

#### Audio Manager Thread  

* check for permission  
* start recording (16000, 16, mono)  
* store recorded audio data  
* stop recording  

#### Message Handler Thread

* communicate with UI

------

## Details

### Communication

1. PC side starts server  
2. Client side searches server  
3. Find server, validate:  
   1. client sends `"AndroidMicCheck"`  
   2. server receives and compare  
   3. server sends `"AndroidMicCheckAck"`  
   4. client receives and compare  
4. Establish socket session  
5. Sync data  

### Bluetooth

* Search server only from paired PC devices  
* Socket is closed from validation step and reconnect  

### Wifi

* Support actual wifi (connected to same network)  
* Or USB tethering  
* Socket is maintained from validation step  

### Service

* Runs as a foreground service  
* Manages notification  
* Upon connection, bluetooth is preferred and tested first  

### UI Activity

* Upon create, communicate with service to get latest UI states  