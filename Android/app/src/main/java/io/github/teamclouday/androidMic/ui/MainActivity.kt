package io.github.teamclouday.androidMic.ui

import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import io.github.teamclouday.androidMic.AndroidMicApp
import io.github.teamclouday.androidMic.ui.home.HomeScreen
import io.github.teamclouday.androidMic.ui.theme.AndroidMicTheme
import io.github.teamclouday.androidMic.ui.utils.rememberWindowInfo
import io.github.teamclouday.androidMic.utils.ignore


class MainActivity : ComponentActivity() {
    private val TAG = "MainActivity"

    private val WAIT_PERIOD = 500L

    val vm: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d(TAG, "onCreate")

        setContent {

            AndroidMicTheme(
                theme = vm.prefs.theme.getAsState().value,
                dynamicColor = vm.prefs.dynamicColor.getAsState().value
            ) {
                // get windowInfo for rotation change
                val windowInfo = rememberWindowInfo()

                HomeScreen(vm, windowInfo)
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        if (intent.extras?.getBoolean("ForegroundServiceBound") == true) {
            Log.d(TAG, "onNewIntent -> ForegroundServiceBound")
            // get variable from application
            vm.refreshAppVariables()
            // get status
            vm.askForStatus()
        }
    }

    override fun onStart() {
        super.onStart()
        Log.d(TAG, "onStart")


        vm.handlerServiceResponse()
        // get variable from application
        vm.refreshAppVariables()

        (application as AndroidMicApp).bindService()
    }


    override fun onStop() {
        super.onStop()
        Log.d(TAG, "onStop")
        vm.mMessengerLooper.quitSafely()
        ignore { vm.handlerThread.join(WAIT_PERIOD) }

        (application as AndroidMicApp).unBindService()
    }

}