package com.example.androidmic.ui

import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.collectAsState
import androidx.lifecycle.SavedStateViewModelFactory
import androidx.lifecycle.ViewModelProvider
import com.example.androidmic.AndroidMicApp
import com.example.androidmic.ui.home.HomeScreen
import com.example.androidmic.ui.theme.AndroidMicTheme
import com.example.androidmic.ui.utils.rememberWindowInfo
import com.example.androidmic.utils.ignore


class MainActivity : ComponentActivity() {
    private val TAG = "MainActivity"

    private val WAIT_PERIOD = 500L

    private lateinit var mainViewModel: MainViewModel

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d(TAG, "onCreate")
        mainViewModel = ViewModelProvider(
            this,
            SavedStateViewModelFactory(application, this)
        )[MainViewModel::class.java]


        setContent {
            val uiStates = mainViewModel.uiStates.collectAsState()

            AndroidMicTheme(
                theme = uiStates.value.theme,
                dynamicColor = uiStates.value.dynamicColor
            ) {
                // get windowInfo for rotation change
                val windowInfo = rememberWindowInfo()

                HomeScreen(mainViewModel, windowInfo)
            }
        }
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        if (intent?.extras?.getBoolean("ForegroundServiceBound") == true) {
            Log.d(TAG, "onNewIntent -> ForegroundServiceBound")
            // get variable from application
            mainViewModel.refreshAppVariables(false)
            // get status
            mainViewModel.askForStatus()
        }
    }

    override fun onStart() {
        super.onStart()
        Log.d(TAG, "onStart")
        mainViewModel.handlerServiceResponse()
        // get variable from application, bind the service if necessary
        mainViewModel.refreshAppVariables(false)

        if (!mainViewModel.mBound) {
            (application as AndroidMicApp).bindService()
        } else {
            // get status
            mainViewModel.askForStatus()
        }
    }


    override fun onStop() {
        super.onStop()
        Log.d(TAG, "onStop")
        mainViewModel.mMessengerLooper.quitSafely()
        ignore { mainViewModel.handlerThread.join(WAIT_PERIOD) }
    }
}