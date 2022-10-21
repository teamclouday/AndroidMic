package com.example.androidmic.ui

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.appcompat.app.AppCompatDelegate
import androidx.compose.runtime.collectAsState
import androidx.core.os.LocaleListCompat
import androidx.lifecycle.SavedStateViewModelFactory
import androidx.lifecycle.ViewModelProvider
import com.example.androidmic.ui.home.HomeScreen
import com.example.androidmic.ui.theme.AndroidMicTheme
import com.example.androidmic.ui.utils.rememberWindowInfo


class MainActivity : ComponentActivity() {
    private val TAG = "MainActivity"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d(TAG, "onCreate")
        val mainViewModel = ViewModelProvider(
            this,
            SavedStateViewModelFactory(application, this)
        )[MainViewModel::class.java]

        val appLocale: LocaleListCompat = LocaleListCompat.forLanguageTags("xx-YY")
        // Call this on the main thread as it may require Activity.restart()
        AppCompatDelegate.setApplicationLocales(appLocale)

        AppCompatDelegate.getApplicationLocales()

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

}