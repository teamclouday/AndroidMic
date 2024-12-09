# Android Microphone

Use your Android phone as a microphone to PC

------

## How to Use

### PC Side

__Start the app__: The installer will be released in the future. For now, make sure rust and cargo are installed on the system and run following command to build the app:
```
cd RustApp
cargo run --release
```

__Pick an output audio device__: You will see a list of audio player devices from the dropdown list. Here you want to choose a device that is wired to the virtual mic device on your system that you will be using.

<details>
<summary>
More about output device
</summary>

The step is system independent.

On Windows you can use [Virtual Audio Cable](https://vac.muzychenko.net/en/download.htm) or [VB Cable](https://vb-audio.com/Cable/). Both software will install virtual input and output audio devices on your system. After that map the output player device to the input mic device so any audio our app played to the device is transferred to the virtual mic device.
</details>

__Choose a connection method__: This is how your phone will be connected to your PC and stream audio from the mic.

For TCP & UDP, connect your phone and PC to the same internet.

For USB serial, connect your phone to PC with a cable.

<details>
<summary>
More about USB serial
</summary>

This option also requires configurations that are system independent.

On Windows, make sure the adb process is shutdown and android studio is closed.

On MacOS, it should just work.

On Linux, you will need to configure [udev](https://github.com/libusb/libusb/wiki/FAQ#can-i-run-libusb-applications-on-linux-without-root-privilege) so that the app has permission to use USB.
</details>

For USB adb, make sure the system has installed [adb](https://developer.android.com/tools/adb). The connect your phone to PC.

__Configure advanced settings__: Click to open the advanced settings window, and pick an audio format the output audio device supports. Usually sample rate of 44.1k or 48k, mono channel, and i16 or i24 are supported.

### Android Side

__Start the app__: Similar to the PC side, the installer will be released in the future. For now, compile the app from Android studio and install on your phone.

__Configure the app__: Open the side drawer menu, configure the connection method according to the option on PC app. Then pick the same audio format as the one in PC app advanced settings.

__Connect__: First start recording and give sufficient permissions. Recording permission for accessing your phone's mic. Notification permission so the app can let you know if it is still recording in the background. Then connect to the PC app.

<details>
<summary>More about connection configurations</summary>

For TCP/UDP, you will need to enter the PC address and port. You can find that information from the log area on PC app.

For USB adb, set your phone to developer mode and enable USB debugging.

For USB serial, make sure your phone's USB setting is charging only. With this option, the app will ask your permission to launch the app in accessory mode.
</details>

--------

For more question / feature request / bug report, please submit issues to ask.

-------

## Some Notes

The PC app started as a WPF app written in C# and was only supported on Windows. Now most of the features are recreated in Rust app thanks to @wiiznokes and it's cross platform supported. But here's the [link to the WPF app branch](https://github.com/teamclouday/AndroidMic/tree/wpf-app-backup) in case you are interested.

Bluetooth is no longer supported because USB serial is made possible.

There is still plan to add some advanced features such as automatic resampling and DSPs for noise cancellation etc.