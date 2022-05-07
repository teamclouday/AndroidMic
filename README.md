# Android Microphone

Use your Android phone as a microphone to Windows PC

------

## Requirements  
* Android phone with bluetooth/wifi  
* Windows PC with bluetooth/wifi  
* Installed [Virtual Audio Cable (VAC)](https://vac.muzychenko.net/en/) on Windows, will hear "trial" voice if your driver is in trial mode  
  I'm actually using [VB-Audio](https://vb-audio.com/Cable/) as alternative now since it is completely free  

------

## How to use  

<details>
<summary>Config Audio Device</summary>

1. Run Windows side app  
2. Select audio speaker from drop down list to the one that VB created  
   <img src="Assets/sound_config1.png" width="300" alt="sound config1">  
3. Use the corresponding microphone created by VB  
   <img src="Assets/sound_config2.png" width="300" alt="sound config2">  
4. In `Properties` of both, make sure both set default format to following:  
   <img src="Assets/sound_config4.png" width="300" alt="sound config4">  
5. For speaker, click `Configure Speakers` and set channel to `Mono`:  
   <img src="Assets/sound_config3.png" width="300" alt="sound config3">  
6. For microphone, click `Properties` and set following:  
   <img src="Assets/sound_config5.png" width="300" alt="sound config5">

On my machine, this setup has the lowest delay and best sound quality. VB is not optimized as hardware devices, so these configurations are important for audio.

</details>

<details>
<summary>Volume Control</summary>

1. Run Windows side app  
2. Drag slider to control volune  

</details>

<details>
<summary>Connection: Bluetooth</summary>

1. Make sure PC and phone are paired once  
2. Check `Bluetooth` button on Windows app  
3. Click `Connect` on Windows app to start server  
4. Click `Connect` on Android app to connect  
5. Tap `Record Audio` on Android app to start transferring audio  

</details>

<details>
<summary>Connection: Wifi</summary>

1. Make sure PC and phone are under the same network  
   1. Can be under the same router with Wifi  
   2. Can have PC connected to ethernet of the same router  
   3. Can have PC connected to phone by cable and enable USB tethering on phone  
   4. USB tethering may not work if it is not the first available network  
2. Click `Connect` on Windows app to start server  
3. Click `Connect` on Android app to connect  
4. Enter `IP` and `Port` (displayed on Windows side) on Android app  
5. Tap `Record Audio` on Android app to start transferring audio  

</details>

------

## Feature Plans  

- [x] Windows app can minimize to system tray  
- [x] Volume control on Windows side  
- [x] Audio visualization on Windows side  
- [x] Show notification when mic is in use on Android side  
- [ ] Implement Acoustic Echo Cancellation (AEC)  
- [ ] Audio effect filters (Not yet released)  
  - [x] Pitch Shifter  
  - [ ] To be continued  

Check out [Avalonia](https://github.com/AvaloniaUI/Avalonia)! With this it may be possible to port all Windows code to .Net Core that can be compiled to support Linux/MacOS!

------

## Releases

Pre-built installers can be found [here](https://github.com/teamclouday/AndroidMic/releases)  


------

## Windows Side

<img src="Assets/p1.png" width="500" alt="Windows Side">

## Android Side (Portrait)

<img src="Assets/p2.jpg" width="250" alt="Android Side">

## Android Side (Landscape)

<img src="Assets/p3.jpg" width="500" alt="Android Side">

