# Android Microphone (Android side)  

Android Microphone Project (Android Application folder)

------

### Structure

Four major threads:  
* UI thread  
* Bluetooth client thread  
* USB tcp client thread  
* Audio recorder thread  

------
#### UI Thread  

* manage the rest 2 threads  
* handle button and switch clicks  
* display messages  

#### Bluetooth Thread  

* check for permission  
* search for valid PC  
* validate connected PC  
* establish connection  
* transfer stored audio data  
* cancel connection  

#### USB Thread  

* check if USB tethering is enabled  
* connect to server based on input address  
* transfer stored audio data  
* cancel connection  

#### Audio Thread  

* check for permission  
* start recording (44100, 16, mono)  
* store recorded audio data  
* stop recording  

------

### Notes  

This is the first Android application I write in Kotlin. The bluetooth part is basically the same as the Java applications I wrote before. However, Kotlin appears to be cleaner and shorter in length. A notable difference is that Kotlin uses `coroutines` to replace `AsyncTask` in Java, which I think is better. Another difference is that Kotlin is strict with `null` values and provide many ways to check them.  

A drawback of this application is that it won't be able to run in background. So when connected, your application will force the screen on and will close connection whenever the app is put into background. A possible fix is to create [Background Services](https://developer.android.com/training/run-background-service/create-service) instead of threads I'm currently using. However, that requires much efforts in learning, and the communication with main activity is especially complex. I will leave it like that.  

USB communication is achieved by enabling USB tethering. In this case, the network through PC can be detected by Android. By establishing a TCP socket, the device can communicate with PC app through USB.  