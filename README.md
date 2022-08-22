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

On my machine, this setup has the lowest delay and best sound quality.  
Can further improve audio latency by opening `VBCABLE_ControlPanel.exe` (from downloaded folder of VB) and set `Max Latency` in Options to 2048 smp:  
<img src="Assets/sound_config6.png" width="600" alt="sound config6">

Do not set to 512 smp since that will cause most buffers lost. If 2048 has no sound or bad quality, consider a higher smp.

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

<details>
<summary>Advanced Filters</summary>

1. Run Windows side app  
2. Connect phone to app  
3. Click `Advanced Effects` to open advanced settings window  
4. Expand a filter and check `enabled` to test effects  

__Pitch Shifter__:  
* Slider controls pitch shift factor  

__White Noise__:  
* Slider controls the noise strength  

__Repeat Track__:  
* Check `Repeat` to enable looped audio track  
* Slider controls audio track volume strength  
* `Select File` to select an audio file to play  

__SpeexDSP Filters__:  
* Check corresponding filters to enable/disable   
* If Echo Cancellation does not work (your friend hears echo from your speaker)  
  Try re-enable echo cancellation to restart cancellation state  
  Echo Cancellation may take a while (5-10s) to optimize  

</details>

------

## Feature Plans  

- [x] Windows app can minimize to system tray  
- [x] Volume control on Windows side  
- [x] Audio visualization on Windows side  
- [x] Show notification when mic is in use on Android side  
- [x] Audio effect filters  
  - [x] Pitch Shifter  
  - [x] Add White Noise  
  - [x] Repeated Background Audio  
  - [x] SpeexDSP Noise Cancellation  
  - [x] SpeexDSP Automatic Gain Control  
  - [x] SpeexDSP Voice Activity Detection  
  - [x] SpeexDSP Echo Cancellation  
- [x] Memorized user settings
- [x] Select network adapter in wifi mode
- [ ] Port all Windows code & dependencies to .Net Core (with [Avalonia](https://github.com/AvaloniaUI/Avalonia))
  - [ ] MacOS Support
  - [ ] Linux Support

------

## Optimization Plans

To further reduce latency, here're the possible areas to optimize:
- [x] Use `Oboe` audio library
- [x] Improve `NAudio` filters
- [x] Use modern `NAudio` WASAPI player (no more Windows XP / lower support)
- [x] Garbage Collection friendly buffers on Android Kotlin
- [ ] Garbage Collection friendly buffers on Windows C#
- [ ] Threads / Coroutines priority level
- [ ] Profile and analyze hot code paths on each

------

## Releases

Pre-built installers can be found [here](https://github.com/teamclouday/AndroidMic/releases)  

------

## Windows Side

<p float="left">
<img src="Assets/p1.png" width="510" alt="Windows Side">
<img src="Assets/p4.png" width="250" alt="Windows Side">
</p>

## Android Side

<img src="Assets/p2.jpg" width="250" alt="Android Side">  

<img src="Assets/p3.jpg" width="500" alt="Android Side">

