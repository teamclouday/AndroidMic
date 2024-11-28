package com.example.androidMic.ui.utils

import android.content.Context


/**
 * Rules: key should be upper case snake case
 * Ex: SAMPLE_RATE
 * The key should match the name in the app state
 */
class AppPreferences(
    context: Context
) : PreferencesManager(context, "settings") {



}
