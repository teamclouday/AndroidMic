package com.example.androidmic.ui.home.dialog

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import com.example.androidmic.R
import com.example.androidmic.ui.Event
import com.example.androidmic.ui.MainViewModel
import com.example.androidmic.utils.Languages.Companion.ENGLISH_LANGUAGE
import com.example.androidmic.utils.Languages.Companion.FRENCH_LANGUAGE
import com.example.androidmic.utils.Languages.Companion.SYSTEM_LANGUAGE
import com.example.androidmic.utils.States
import java.util.*

data class LanguageItem(
    val id: Int,
    val title: String,
    val contentDescription: String,
    val onClick: (Int) -> Unit
)

@Composable
fun DialogLanguage(mainViewModel: MainViewModel, uiStates: States.UiStates) {

    if(uiStates.dialogLanguageIsVisible) {
        Dialog(
            onDismissRequest = { mainViewModel.onEvent(Event.DismissDialog(R.string.drawerLanguage)) }
        ) {
            Surface(
                shape = MaterialTheme.shapes.medium,
                color = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            ) {

                val listItems = listOf(
                    LanguageItem(
                        id = SYSTEM_LANGUAGE,
                        title = stringResource(id = R.string.system_language),
                        contentDescription = Locale.getDefault().displayLanguage,
                        onClick = {
                            mainViewModel.onEvent(Event.SetLanguage(it))
                        }
                    ),
                    LanguageItem(
                        id = ENGLISH_LANGUAGE,
                        title = stringResource(id = R.string.english_language),
                        contentDescription = stringResource(id = R.string.english_language_native),
                        onClick = {
                            mainViewModel.onEvent(Event.SetLanguage(it))
                        }
                    ),
                    LanguageItem(
                        id = FRENCH_LANGUAGE,
                        title = stringResource(id = R.string.french_language),
                        contentDescription = stringResource(id = R.string.french_language_native),
                        onClick = {
                            mainViewModel.onEvent(Event.SetLanguage(it))
                        }
                    )
                )

                LazyColumn {
                    var count = 1
                    items(listItems) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable {
                                    it.onClick(it.id)
                                }
                        ) {
                            Column {
                                Text(
                                    modifier = Modifier.padding(5.dp),
                                    text = it.title,
                                    style = MaterialTheme.typography.labelLarge,
                                    color = MaterialTheme.colorScheme.onSurface
                                )
                                Text(
                                    modifier = Modifier.padding(start = 5.dp, end = 5.dp, bottom = 5.dp),
                                    text = it.contentDescription,
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSurface
                                )
                                if(count != listItems.size)
                                    Divider(
                                        color = MaterialTheme.colorScheme.onSurface
                                    )
                                count++
                            }
                        }
                    }
                }
            }
        }
    }
}
