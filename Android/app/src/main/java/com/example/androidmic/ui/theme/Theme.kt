package com.example.androidmic.ui.theme


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


private val DarkColorScheme = darkColorScheme(
    primary = Purple500,
    onPrimary = Color.White,

    secondary = Teal700,
    onSecondary = Color.Black,

    tertiary = Color.Black,
    onTertiary = Color.White,

    surface = Color(0xFFFFFBFE),
    onSurface = Color(0xFF1C1B1F),

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

    surface = Purple500,
    onSurface = Color.White,

    background = Color.White,
    onBackground = Color.Black,
)

@Composable
fun AndroidMicTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    // Dynamic color is available on Android 12+
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {
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
