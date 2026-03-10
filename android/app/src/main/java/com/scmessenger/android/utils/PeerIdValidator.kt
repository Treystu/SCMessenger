package com.scmessenger.android.utils

object PeerIdValidator {
    private val PEER_ID_REGEX = Regex("[a-f0-9]{64}")
    
    fun validate(id: String): Boolean = 
        normalize(id).matches(PEER_ID_REGEX)
    
    fun normalize(id: String): String = 
        id.trim().lowercase()
    
    fun isSame(id1: String, id2: String): Boolean =
        normalize(id1) == normalize(id2)
}
