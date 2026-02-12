package com.scmessenger.android.utils

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import com.scmessenger.android.R
import com.scmessenger.android.data.MeshRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import timber.log.Timber

/**
 * BroadcastReceiver for handling share intents.
 * 
 * Receives Intent.ACTION_SEND/ACTION_SEND_MULTIPLE and presents a contact picker
 * to encrypt and queue the shared content as a mesh message.
 * 
 * Features:
 * - Text/plain sharing support
 * - Contact picker dialog
 * - Encryption via MeshRepository
 * - Background message queueing
 */
class ShareReceiver : BroadcastReceiver() {
    
    private val scope = CoroutineScope(Dispatchers.Main + SupervisorJob())
    
    override fun onReceive(context: Context, intent: Intent) {
        when (intent.action) {
            Intent.ACTION_SEND -> handleSingleShare(context, intent)
            Intent.ACTION_SEND_MULTIPLE -> handleMultipleShare(context, intent)
            else -> Timber.w("Unhandled intent action: ${intent.action}")
        }
    }
    
    /**
     * Handle single item share (text).
     */
    private fun handleSingleShare(context: Context, intent: Intent) {
        val type = intent.type
        if (type == null) {
            Timber.w("Share intent has no type")
            return
        }
        
        when {
            type == "text/plain" -> {
                val sharedText = intent.getStringExtra(Intent.EXTRA_TEXT)
                if (sharedText != null) {
                    showContactPicker(context, sharedText)
                } else {
                    Timber.w("Shared text is null")
                }
            }
            else -> {
                Toast.makeText(context, "Unsupported share type: $type", Toast.LENGTH_SHORT).show()
                Timber.w("Unsupported share type: $type")
            }
        }
    }
    
    /**
     * Handle multiple items share (currently not supported).
     */
    private fun handleMultipleShare(context: Context, intent: Intent) {
        Toast.makeText(context, "Multiple items sharing not yet supported", Toast.LENGTH_SHORT).show()
        Timber.w("Multiple share not implemented")
    }
    
    /**
     * Show contact picker dialog to select recipient.
     */
    private fun showContactPicker(context: Context, content: String) {
        try {
            val repository = MeshRepository(context.applicationContext)
            val contacts = repository.listContacts()
            
            if (contacts.isEmpty()) {
                Toast.makeText(context, "No contacts available", Toast.LENGTH_SHORT).show()
                Timber.w("No contacts to share with")
                return
            }
            
            val contactNames = contacts.map { contact ->
                contact.nickname ?: contact.peerId.take(8)
            }.toTypedArray()
            
            AlertDialog.Builder(context)
                .setTitle("Share to Contact")
                .setItems(contactNames) { dialog, which ->
                    val selectedContact = contacts[which]
                    sendMessageToContact(context, repository, selectedContact.peerId, content)
                    dialog.dismiss()
                }
                .setNegativeButton(R.string.cancel) { dialog, _ ->
                    dialog.dismiss()
                }
                .show()
        } catch (e: Exception) {
            Timber.e(e, "Failed to show contact picker")
            Toast.makeText(context, "Failed to load contacts", Toast.LENGTH_SHORT).show()
        }
    }
    
    /**
     * Encrypt and queue message via MeshRepository.
     */
    private fun sendMessageToContact(
        context: Context,
        repository: MeshRepository,
        peerId: String,
        content: String
    ) {
        scope.launch {
            try {
                repository.sendMessage(peerId, content)
                Toast.makeText(context, "Message queued for sending", Toast.LENGTH_SHORT).show()
                Timber.i("Shared message queued for $peerId")
            } catch (e: Exception) {
                Timber.e(e, "Failed to send shared message")
                Toast.makeText(context, "Failed to send message", Toast.LENGTH_SHORT).show()
            }
        }
    }
}
