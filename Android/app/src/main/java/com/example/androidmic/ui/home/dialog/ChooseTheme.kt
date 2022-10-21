package com.example.androidmic.ui.home.dialog

import androidx.compose.foundation.layout.*
import androidx.compose.material3.Checkbox
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
import com.example.androidmic.R
import com.example.androidmic.ui.Event
import com.example.androidmic.ui.MainViewModel
import com.example.androidmic.ui.components.ManagerButton
import com.example.androidmic.ui.components.ManagerCheckBox
import com.example.androidmic.utils.States
import com.example.androidmic.utils.Themes.Companion.DARK_THEME
import com.example.androidmic.utils.Themes.Companion.LIGHT_THEME
import com.example.androidmic.utils.Themes.Companion.SYSTEM_THEME

@Composable
fun DialogTheme(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    val tempSystemTheme = remember {
        mutableStateOf(uiStates.theme == SYSTEM_THEME)
    }
    val tempLightTheme = remember {
        mutableStateOf(uiStates.theme == LIGHT_THEME)
    }
    val tempDarkTheme = remember {
        mutableStateOf(uiStates.theme == DARK_THEME)
    }

    val tempDynamicColor = remember {
        mutableStateOf(uiStates.dynamicColor)
    }

    if (uiStates.dialogThemeIsVisible) {
        Dialog(
            onDismissRequest = {
                tempSystemTheme.value = uiStates.theme == SYSTEM_THEME
                tempLightTheme.value = uiStates.theme == LIGHT_THEME
                tempDarkTheme.value = uiStates.theme == DARK_THEME
                tempDynamicColor.value = uiStates.dynamicColor
                mainViewModel.onEvent(Event.DismissDialog(R.string.drawerTheme))
            }
        ) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {
                Column (
                    horizontalAlignment = Alignment.CenterHorizontally
                ){
                    Spacer(modifier = Modifier.height(10.dp))

                    // system
                    ManagerCheckBox(
                        checked = tempSystemTheme.value,
                        onClick =  {
                            tempSystemTheme.value = true
                            tempLightTheme.value = false
                            tempDarkTheme.value = false
                            mainViewModel.onEvent(
                                Event.SetTheme(SYSTEM_THEME)
                            )

                        },
                        text = stringResource(id = R.string.system_theme)
                    )

                    Spacer(modifier = Modifier.height(10.dp))

                    // light
                    ManagerCheckBox(
                        checked = tempLightTheme.value,
                        onClick =  {
                            tempSystemTheme.value = false
                            tempLightTheme.value = true
                            tempDarkTheme.value = false
                            mainViewModel.onEvent(
                                Event.SetTheme(LIGHT_THEME)
                            )
                        },
                        text = stringResource(id = R.string.light_theme)
                    )

                    Spacer(modifier = Modifier.height(10.dp))

                    // dark
                    ManagerCheckBox(
                        checked = tempDarkTheme.value,
                        onClick =  {
                            tempSystemTheme.value = false
                            tempLightTheme.value = false
                            tempDarkTheme.value = true
                            mainViewModel.onEvent(
                                Event.SetTheme(DARK_THEME)
                            )
                        },
                        text = stringResource(id = R.string.dark_theme)
                    )

                    Divider(
                        modifier = Modifier.padding(10.dp),
                        color = MaterialTheme.colorScheme.onSurface
                    )

                    // dynamic color
                    ManagerCheckBox(
                        checked = tempDynamicColor.value,
                        onClick =  {
                            tempDynamicColor.value = it
                            mainViewModel.onEvent(
                                Event.SetDynamicColor(it)
                            )
                        },
                        text = stringResource(id = R.string.dynamic_color)
                    )
                    
                    Spacer(modifier = Modifier.height(10.dp))
                }
            }
        }
    }
}
