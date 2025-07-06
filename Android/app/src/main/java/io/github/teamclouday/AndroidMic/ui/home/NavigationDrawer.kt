package io.github.teamclouday.AndroidMic.ui.home

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.DarkMode
import androidx.compose.material.icons.rounded.Settings
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
import io.github.teamclouday.AndroidMic.Mode
import io.github.teamclouday.AndroidMic.R
import io.github.teamclouday.AndroidMic.ui.MainViewModel
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogAudioFormat
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogChannelCount
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogIpPort
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogMode
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogSampleRate
import io.github.teamclouday.AndroidMic.ui.home.dialog.DialogTheme


@Composable
fun DrawerBody(vm: MainViewModel) {

    Column(
        modifier = Modifier
            .verticalScroll(rememberScrollState())
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
            Mode.WIFI, Mode.UDP -> {
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

            else -> {

            }

//            Modes.USB, Modes.BLUETOOTH -> {
//
//            }
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
            .clickable {
                expanded.value = true
            }
            .padding(16.dp),
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