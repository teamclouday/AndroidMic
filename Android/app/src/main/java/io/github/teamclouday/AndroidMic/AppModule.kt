package io.github.teamclouday.AndroidMic

import android.content.Context
import io.github.teamclouday.AndroidMic.ui.utils.UiHelper


interface AppModule {
    val appPreferences: AppPreferences
    val uiHelper: UiHelper
    val context: Context

}

class AppModuleImpl(
    override val context: Context
) : AppModule {
    override val appPreferences: AppPreferences by lazy {
        AppPreferences(context)
    }
    override val uiHelper: UiHelper by lazy {
        UiHelper(context)
    }
}