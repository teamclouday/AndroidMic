package io.github.teamclouday.androidMic.ui.theme


import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import io.github.teamclouday.androidMic.Themes


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
    theme: Themes,
    // Dynamic color is available on Android 12+
    dynamicColor: Boolean,
    content: @Composable () -> Unit
) {
    val darkTheme = when (theme) {
        Themes.System -> isSystemInDarkTheme()
        Themes.Light -> false
        Themes.Dark -> true
    }
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }

        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
