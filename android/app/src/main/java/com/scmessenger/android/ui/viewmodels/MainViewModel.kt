package com.scmessenger.android.ui.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.scmessenger.android.data.MeshRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class MainViewModel @Inject constructor(
    private val meshRepository: MeshRepository
) : ViewModel() {

    private val _isReady = MutableStateFlow(false)
    val isReady = _isReady.asStateFlow()

    init {
        checkIdentity()
    }

    private fun checkIdentity() {
        viewModelScope.launch {
            if (meshRepository.isIdentityInitialized()) {
                _isReady.value = true
            } else {
                // Stay not ready, waiting for onboarding
                _isReady.value = false
            }
        }
    }

    fun createIdentity() {
        viewModelScope.launch {
            try {
                meshRepository.createIdentity()
                _isReady.value = true
            } catch (e: Exception) {
                // Handle error
            }
        }
    }
}
