package io.github.teamclouday.androidMic.ui.utils

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

@Composable
fun rememberWindowInfo(): WindowInfo {
    val windowInfo = LocalWindowInfo.current
    val size = windowInfo.containerSize

    val widthDp = with(LocalDensity.current) { size.width.toDp() }
    val heightDp = with(LocalDensity.current) { size.height.toDp() }

    return WindowInfo(
        screenWidthInfo = when {
            widthDp < 600.dp -> WindowInfo.WindowType.Compact
            widthDp < 840.dp -> WindowInfo.WindowType.Medium
            else -> WindowInfo.WindowType.Expanded
        },
        screenHeightInfo = when {
            heightDp < 480.dp -> WindowInfo.WindowType.Compact
            heightDp < 900.dp -> WindowInfo.WindowType.Medium
            else -> WindowInfo.WindowType.Expanded
        },
        screenWidth = widthDp,
        screenHeight = heightDp
    )
}

data class WindowInfo(
    val screenWidthInfo: WindowType,
    val screenHeightInfo: WindowType,
    val screenWidth: Dp,
    val screenHeight: Dp
) {
    sealed class WindowType {
        object Compact : WindowType()
        object Medium : WindowType()
        object Expanded : WindowType()
    }
}