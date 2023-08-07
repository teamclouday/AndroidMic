package com.example.androidMic.ui.home.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidMic.R
import com.example.androidMic.ui.Event
import com.example.androidMic.ui.MainViewModel
import com.example.androidMic.ui.components.ManagerCheckBox
import com.example.androidMic.ui.States
import com.example.androidMic.ui.Themes

@Composable
fun DialogTheme(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    val tempSystemTheme = remember {
        mutableStateOf(uiStates.theme == Themes.SYSTEM)
    }
    val tempLightTheme = remember {
        mutableStateOf(uiStates.theme == Themes.LIGHT)
    }
    val tempDarkTheme = remember {
        mutableStateOf(uiStates.theme == Themes.DARK)
    }

    val tempDynamicColor = remember {
        mutableStateOf(uiStates.dynamicColor)
    }

    if (uiStates.dialogThemeIsVisible) {
        Dialog(
            onDismissRequest = {
                tempSystemTheme.value = uiStates.theme == Themes.SYSTEM
                tempLightTheme.value = uiStates.theme == Themes.LIGHT
                tempDarkTheme.value = uiStates.theme == Themes.DARK
                tempDynamicColor.value = uiStates.dynamicColor
                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerTheme))
            }
        ) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Spacer(modifier = Modifier.height(25.dp))

                    // system
                    ManagerCheckBox(
                        checked = tempSystemTheme.value,
                        onClick = {
                            tempSystemTheme.value = true
                            tempLightTheme.value = false
                            tempDarkTheme.value = false
                            mainViewModel.onEvent(
                                Event.SetTheme(Themes.SYSTEM)
                            )

                        },
                        text = stringResource(id = R.string.system_theme)
                    )

                    Spacer(modifier = Modifier.height(20.dp))

                    // light
                    ManagerCheckBox(
                        checked = tempLightTheme.value,
                        onClick = {
                            tempSystemTheme.value = false
                            tempLightTheme.value = true
                            tempDarkTheme.value = false
                            mainViewModel.onEvent(
                                Event.SetTheme(Themes.LIGHT)
                            )
                        },
                        text = stringResource(id = R.string.light_theme)
                    )

                    Spacer(modifier = Modifier.height(20.dp))

                    // dark
                    ManagerCheckBox(
                        checked = tempDarkTheme.value,
                        onClick = {
                            tempSystemTheme.value = false
                            tempLightTheme.value = false
                            tempDarkTheme.value = true
                            mainViewModel.onEvent(
                                Event.SetTheme(Themes.DARK)
                            )
                        },
                        text = stringResource(id = R.string.dark_theme)
                    )

                    Divider(
                        modifier = Modifier.padding(20.dp),
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    // dynamic color
                    ManagerCheckBox(
                        checked = tempDynamicColor.value,
                        onClick = {
                            tempDynamicColor.value = it
                            mainViewModel.onEvent(
                                Event.SetDynamicColor(it)
                            )
                        },
                        text = stringResource(id = R.string.dynamic_color)
                    )

                    Spacer(modifier = Modifier.height(25.dp))
                }
            }
        }
    }
}
