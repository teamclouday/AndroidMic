package com.example.androidMic.ui.home

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidMic.R
import com.example.androidMic.ui.Dialogs
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.Modes
import com.example.androidMic.ui.components.ManagerSetting
import com.example.androidMic.ui.home.dialog.DialogMode
import com.example.androidMic.ui.home.dialog.DialogTheme
import com.example.androidMic.ui.home.dialog.DialogWifiIpPort
import com.example.androidMic.ui.States
import com.example.androidMic.ui.Themes

data class MenuItem(
    val id: Dialogs,
    val title: String,
    val subTitle: String,
    val contentDescription: String,
    val icon: Int
)

@Composable
fun DrawerBody(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    DialogWifiIpPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogMode(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogTheme(mainViewModel = mainViewModel, uiStates = uiStates)


    val connectionItems = listOf(
        MenuItem(
            id = Dialogs.Modes,
            title = stringResource(id = R.string.drawerMode),
            subTitle = when (uiStates.mode) {
                Modes.WIFI -> stringResource(id = R.string.mode_wifi)
                Modes.BLUETOOTH -> stringResource(id = R.string.mode_bluetooth)
                Modes.USB -> stringResource(id = R.string.mode_usb)
                Modes.UDP -> stringResource(id = R.string.mode_udp)
            },
            contentDescription = "set mode",
            icon = R.drawable.settings_24px
        ),
        MenuItem(
            id = Dialogs.IpPort,
            title = stringResource(id = R.string.drawerWifiIpPort),
            subTitle = uiStates.ip + ":" + uiStates.port,
            contentDescription = "set ip and port",
            icon = R.drawable.wifi_24px

        ),
    )

    val otherItems = listOf(
        MenuItem(
            id = Dialogs.Themes,
            title = stringResource(id = R.string.drawerTheme),
            subTitle = when (uiStates.theme) {
                Themes.SYSTEM -> stringResource(id = R.string.system_theme)
                Themes.LIGHT -> stringResource(id = R.string.light_theme)
                Themes.DARK -> stringResource(id = R.string.dark_theme)
            },
            contentDescription = "set theme",
            icon = R.drawable.dark_mode_24px
        )
    )

    LazyColumn(
        modifier = Modifier
            .fillMaxHeight()
            .width(355.dp)
            .background(color = MaterialTheme.colorScheme.background)
    ) {
        // setting title
        item {
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
        }

        // connection subtitle
        item {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(10.dp)
            ) {
                Text(
                    text = stringResource(id = R.string.drawer_subtitle_connection),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onBackground
                )
            }
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }

        items(connectionItems) { item ->
            ManagerSetting(mainViewModel, item)
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }

        item {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 10.dp, top = 25.dp, bottom = 10.dp)
            ) {
                Text(
                    text = stringResource(id = R.string.drawer_subtitle_other),
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onBackground
                )
            }
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }
        items(otherItems) { item ->
            ManagerSetting(mainViewModel, item)
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }
    }
}