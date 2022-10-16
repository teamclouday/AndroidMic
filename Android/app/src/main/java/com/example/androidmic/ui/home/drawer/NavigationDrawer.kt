package com.example.androidmic.ui.home.drawer

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.example.androidmic.R
import com.example.androidmic.ui.Event
import com.example.androidmic.ui.MainViewModel
import com.example.androidmic.ui.home.DialogIpPort
import com.example.androidmic.ui.home.DialogMode
import com.example.androidmic.utils.States

@Composable
fun DrawerHeader() {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 64.dp).padding(start = 30.dp)
    ) {
        Text(text = stringResource(id = R.string.drawerHeader),
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onBackground)
    }
}

@Composable
fun DrawerBody(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    DialogIpPort(mainViewModel = mainViewModel, uiStates = uiStates)
    DialogMode(mainViewModel = mainViewModel, uiStates = uiStates)

    DrawerBodyList(
        items = listOf(
            MenuItem(
                id = R.string.drawerIpPort,
                title = stringResource(id = R.string.drawerIpPort),
                contentDescription = "set ip and port",
                icon = painterResource(id = R.drawable.ic_baseline_wifi_24)

            ),
            MenuItem(
                id = R.string.drawerMode,
                title = stringResource(id = R.string.drawerMode),
                contentDescription = "set mode",
                icon = rememberVectorPainter(Icons.Default.Settings)
            )
        ),
        onItemClick = {
            mainViewModel.onEvent(Event.ShowDialog(it.id))
        },
        uiStates = uiStates
    )
}


@Composable
fun DrawerBodyList(
    items: List<MenuItem>,
    onItemClick: (MenuItem) -> Unit,
    modifier: Modifier = Modifier,
    uiStates: States.UiStates
) {
    LazyColumn(modifier) {
        items(items) { item ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable {
                        onItemClick(item)
                    }
                    .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
            ){
                Icon(
                    painter = item.icon,
                    contentDescription = item.contentDescription,
                    tint = MaterialTheme.colorScheme.onBackground
                )
                Spacer(modifier = Modifier.width(16.dp))
                Column {
                    Text(
                        text = item.title,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.onBackground
                    )

                    when (item.id) {
                        R.string.drawerIpPort -> {
                            Text(
                                text = uiStates.IP + ":" + uiStates.PORT,
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onBackground
                            )
                        }
                        R.string.drawerMode -> {
                            Text(
                                text = uiStates.textMode,
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onBackground
                            )
                        }
                    }
                }
            }
            Divider(color = MaterialTheme.colorScheme.onBackground)
        }
    }
}