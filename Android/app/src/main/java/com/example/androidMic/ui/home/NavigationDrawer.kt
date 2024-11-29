package com.example.androidMic.ui.home

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.DarkMode
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material.icons.rounded.Usb
import androidx.compose.material.icons.rounded.Wifi
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidMic.Modes
import com.example.androidMic.R
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.home.dialog.DialogAudioFormat
import com.example.androidMic.ui.home.dialog.DialogChannelCount
import com.example.androidMic.ui.home.dialog.DialogMode
import com.example.androidMic.ui.home.dialog.DialogSampleRate
import com.example.androidMic.ui.home.dialog.DialogTheme
import com.example.androidMic.ui.home.dialog.DialogIpPort
import com.example.androidMic.ui.home.dialog.DialogPort

data class MenuItem(
    val title: String,
    val subTitle: String,
    val contentDescription: String,
    val icon: ImageVector? = null
)

@Composable
fun DrawerBody(vm: MainViewModel) {

    Column(
        modifier = Modifier
            .fillMaxHeight()
            .width(355.dp)
            .background(color = MaterialTheme.colorScheme.background)
    ) {
        // setting title
        Box(
            modifier = Modifier
                .padding(vertical = 64.dp)
                .padding(start = 25.dp)
        ) {
            Text(
                text = stringResource(id = R.string.drawerHeader),
                style = MaterialTheme.typography.titleLarge,
                color = MaterialTheme.colorScheme.onBackground
            )
        }

        // Connection
        SettingsItemsSubtitle(R.string.drawer_subtitle_connection)

        val dialogModeExpanded = rememberSaveable {
            mutableStateOf(false)
        }

        val mode = vm.prefs.mode.getAsState()

        DialogMode(vm = vm, expanded = dialogModeExpanded)
        SettingsItem(
            title = stringResource(id = R.string.drawerMode),
            subTitle = mode.value.toString(),
            contentDescription = "set mode",
            icon = Icons.Rounded.Settings,
            expanded = dialogModeExpanded
        )

        when (mode.value) {
            Modes.WIFI, Modes.UDP -> {
                val dialogIpPortExpanded = rememberSaveable {
                    mutableStateOf(false)
                }
                DialogIpPort(vm = vm, expanded = dialogIpPortExpanded)
                SettingsItem(
                    title = stringResource(id = R.string.drawerIpPort),
                    subTitle = vm.prefs.ip.getAsState().value + ":" + vm.prefs.port.getAsState().value,
                    contentDescription = "set ip and port",
                    icon = Icons.Rounded.Wifi,
                    expanded = dialogIpPortExpanded
                )
            }

            Modes.USB -> {
                val dialogPortExpanded = rememberSaveable {
                    mutableStateOf(false)
                }
                DialogPort(vm = vm, expanded = dialogPortExpanded)
                SettingsItem(
                    title = stringResource(id = R.string.dialog_port),
                    subTitle = vm.prefs.port.getAsState().value,
                    contentDescription = "set port",
                    icon = Icons.Rounded.Usb,
                    expanded = dialogPortExpanded
                )
            }

            Modes.BLUETOOTH -> {

            }
        }

        // Audio
        SettingsItemsSubtitle(R.string.drawer_subtitle_record)

        val dialogSampleRateExpanded = rememberSaveable {
            mutableStateOf(false)
        }
        DialogSampleRate(vm = vm, expanded = dialogSampleRateExpanded)
        SettingsItem(
            title = stringResource(id = R.string.sample_rate),
            subTitle = vm.prefs.sampleRate.getAsState().value.value.toString(),
            contentDescription = "set sample rate",
            expanded = dialogSampleRateExpanded
        )

        val dialogChannelCountExpanded = rememberSaveable {
            mutableStateOf(false)
        }
        DialogChannelCount(vm = vm, expanded = dialogChannelCountExpanded)
        SettingsItem(
            title = stringResource(id = R.string.channel_count),
            subTitle = vm.prefs.channelCount.getAsState().value.toString(),
            contentDescription = "set channel count",
            expanded = dialogChannelCountExpanded
        )

        val dialogAudioFormatExpanded = rememberSaveable {
            mutableStateOf(false)
        }
        DialogAudioFormat(vm = vm, expanded = dialogAudioFormatExpanded)
        SettingsItem(
            title = stringResource(id = R.string.audio_format),
            subTitle = vm.prefs.audioFormat.getAsState().value.toString(),
            contentDescription = "set audio format",
            expanded = dialogAudioFormatExpanded
        )

        // Other
        SettingsItemsSubtitle(R.string.drawer_subtitle_other)

        val dialogThemesExpanded = rememberSaveable {
            mutableStateOf(false)
        }
        DialogTheme(vm = vm, expanded = dialogThemesExpanded)
        SettingsItem(
            title = stringResource(id = R.string.drawerTheme),
            subTitle = vm.prefs.theme.getAsState().value.toString(),
            contentDescription = "set theme",
            icon = Icons.Rounded.DarkMode,
            expanded = dialogThemesExpanded
        )

    }
}


@Composable
private fun SettingsItemsSubtitle(
    subtitle: Int
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(start = 10.dp, top = 25.dp, bottom = 10.dp)
    ) {
        Text(
            text = stringResource(id = subtitle),
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onBackground
        )
    }
    HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
}

@Composable
private fun SettingsItem(
    title: String,
    subTitle: String,
    contentDescription: String,
    icon: ImageVector? = null,
    expanded: MutableState<Boolean>
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(16.dp)
            .clickable {
                expanded.value = true
            },
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (icon != null) {
            Icon(
                imageVector = icon,
                contentDescription = contentDescription,
                tint = MaterialTheme.colorScheme.onBackground
            )
        }
        Spacer(modifier = Modifier.width(16.dp))
        Column {
            Text(
                text = title,
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onBackground
            )

            Text(
                text = subTitle,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onBackground
            )
        }
    }
    HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
}