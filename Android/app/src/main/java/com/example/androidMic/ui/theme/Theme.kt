package com.example.androidMic.ui.theme


import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat
import com.example.androidMic.utils.Themes.Companion.DARK_THEME
import com.example.androidMic.utils.Themes.Companion.LIGHT_THEME
import com.example.androidMic.utils.Themes.Companion.SYSTEM_THEME


private val DarkColorScheme = darkColorScheme(
    primary = Purple500,
    onPrimary = Color.White,

    secondary = Teal200,
    onSecondary = Color.Black,

    tertiary = Color.Black,
    onTertiary = Color.White,

    surface = DarkGrey,
    onSurface = Color.White,

    background = Color.Black,
    onBackground = Color.White,
)

private val LightColorScheme = lightColorScheme(
    primary = Purple500,
    onPrimary = Color.White,

    secondary = Teal200,
    onSecondary = Color.Black,

    tertiary = Color.Black,
    onTertiary = Color.White,

    surface = Color.White,
    onSurface = Color.Black,

    background = Color.White,
    onBackground = Color.Black,
)

@Composable
fun AndroidMicTheme(
    theme: Int,
    // Dynamic color is available on Android 12+
    dynamicColor: Boolean,
    content: @Composable () -> Unit
) {
    val darkTheme = when (theme) {
        SYSTEM_THEME -> isSystemInDarkTheme()
        LIGHT_THEME -> false
        DARK_THEME -> true
        else -> isSystemInDarkTheme()
    }
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }
    val view = LocalView.current
    if (!view.isInEditMode) {
        val currentWindow = (view.context as? Activity)?.window
            ?: throw Exception("Not in an activity - unable to get Window reference")

        SideEffect {
            /* the default code did the same cast here - might as well use our new variable! */
            currentWindow.statusBarColor = colorScheme.primary.toArgb()
            /* accessing the insets controller to change appearance of the status bar, with 100% less deprecation warnings */
            WindowCompat.getInsetsController(currentWindow, view).isAppearanceLightStatusBars =
                darkTheme
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
