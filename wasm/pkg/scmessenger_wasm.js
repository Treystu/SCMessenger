/* @ts-self-types="./scmessenger_wasm.d.ts" */

export class IronCore {
    static __wrap(ptr) {
        const obj = Object.create(IronCore.prototype);
        obj.__wbg_ptr = ptr;
        IronCoreFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IronCoreFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ironcore_free(ptr, 0);
    }
    /**
     * Compute a BLAKE3 hash of the given data.
     * Returns the 32-byte hash as a hex-encoded string.
     * @param {Uint8Array} data
     * @returns {string}
     */
    blake3Hash(data) {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_blake3Hash(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred2_0 = r0;
            deferred2_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Block a peer AND delete all their stored messages (cascade purge).
     * Future payloads from this peer are dropped at the ingress layer.
     * @param {string} peer_id
     * @param {string | null} [reason]
     */
    blockAndDeletePeer(peer_id, reason) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(reason) ? 0 : passStringToWasm0(reason, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            wasm.ironcore_blockAndDeletePeer(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Block a peer by ID, with an optional reason string.
     * @param {string} peer_id
     * @param {string | null} [reason]
     */
    blockPeer(peer_id, reason) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(reason) ? 0 : passStringToWasm0(reason, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            wasm.ironcore_blockPeer(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the count of blocked peers.
     * @returns {number}
     */
    blockedCount() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_blockedCount(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {any} js_message
     * @param {any} js_ui_state
     * @returns {any}
     */
    classifyNotification(js_message, js_ui_state) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_classifyNotification(retptr, this.__wbg_ptr, addHeapObject(js_message), addHeapObject(js_ui_state));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Return the federated nickname for a contact (the nickname advertised by the peer).
     * Returns None if the contact is not found or has no federated nickname set.
     * @param {string} peer_id
     * @returns {string | undefined}
     */
    contactFederatedNickname(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_contactFederatedNickname(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v2;
            if (r0 !== 0) {
                v2 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Dial a remote peer by multiaddr.
     * @param {string} multiaddr
     * @returns {Promise<void>}
     */
    dial(multiaddr) {
        const ptr0 = passStringToWasm0(multiaddr, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_dial(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Drain and return all messages that have arrived since the last call.
     *
     * Returns a `js_sys::Array` of plain JS objects with the same shape as
     * the object returned by `receiveMessage`:
     * `{ id, senderId, text, timestamp }`.
     *
     * The internal buffer is cleared on each call; messages are not duplicated
     * across successive calls.
     * @returns {Array<any>}
     */
    drainReceivedMessages() {
        const ret = wasm.ironcore_drainReceivedMessages(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Promise<string>}
     */
    exportDiagnostics() {
        const ret = wasm.ironcore_exportDiagnostics(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Export the local identity as a backup string (for import on another device).
     * @param {string} passphrase
     * @returns {string}
     */
    exportIdentityBackup(passphrase) {
        let deferred3_0;
        let deferred3_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(passphrase, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_exportIdentityBackup(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr2 = r0;
            var len2 = r1;
            if (r3) {
                ptr2 = 0; len2 = 0;
                throw takeObject(r2);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Export all recorded log entries as a single string.
     * @returns {string}
     */
    exportLogs() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_exportLogs(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * Derive the Ed25519 public key hex from a libp2p PeerId string.
     * @param {string} peer_id
     * @returns {string}
     */
    extractPublicKeyFromPeerId(peer_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_extractPublicKeyFromPeerId(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr2 = r0;
            var len2 = r1;
            if (r3) {
                ptr2 = 0; len2 = 0;
                throw takeObject(r2);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Flush the outbox for a specific peer, returning the count of messages drained.
     * The caller should then send each message via the swarm transport.
     * @param {string} peer_id
     * @returns {number}
     */
    flushOutboxForPeer(peer_id) {
        const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_flushOutboxForPeer(this.__wbg_ptr, ptr0, len0);
        return ret >>> 0;
    }
    /**
     * @param {bigint} since
     * @returns {any}
     */
    getAuditEventsSince(since) {
        const ret = wasm.ironcore_getAuditEventsSince(this.__wbg_ptr, since);
        return takeObject(ret);
    }
    /**
     * @returns {any}
     */
    getAuditLog() {
        const ret = wasm.ironcore_getAuditLog(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Promise<string>}
     */
    getConnectionPathState() {
        const ret = wasm.ironcore_getConnectionPathState(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {WasmContactManager}
     */
    getContactManager() {
        const ret = wasm.ironcore_getContactManager(this.__wbg_ptr);
        return WasmContactManager.__wrap(ret);
    }
    /**
     * Get the current daemon socket URL.
     * @returns {any}
     */
    getDaemonSocketUrl() {
        const ret = wasm.ironcore_getDaemonSocketUrl(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Return the default settings for the Web platform.
     * @returns {any}
     */
    getDefaultSettings() {
        const ret = wasm.ironcore_getDefaultSettings(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get the installation-local device ID (UUIDv4).
     * @returns {string | undefined}
     */
    getDeviceId() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_getDeviceId(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get current disk usage statistics.
     * @returns {any}
     */
    getDiskStats() {
        const ret = wasm.ironcore_getDiskStats(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {string}
     */
    getDriftState() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_getDriftState(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {number}
     */
    getDriftStoreSize() {
        const ret = wasm.ironcore_getDriftStoreSize(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {string} peer_id
     * @returns {any}
     */
    getEnhancedPeerReputation(peer_id) {
        const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_getEnhancedPeerReputation(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Get externally observed addresses (if available) for diagnostics parity.
     * @returns {Promise<any>}
     */
    getExternalAddresses() {
        const ret = wasm.ironcore_getExternalAddresses(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {WasmHistoryManager}
     */
    getHistoryManager() {
        const ret = wasm.ironcore_getHistoryManager(this.__wbg_ptr);
        return WasmHistoryManager.__wrap(ret);
    }
    /**
     * Get identity info from the local IronCore.
     * In Daemon mode, this may return a placeholder until the daemon provides identity.
     * @returns {any}
     */
    getIdentityFromDaemon() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_getIdentityFromDaemon(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {any}
     */
    getIdentityInfo() {
        const ret = wasm.ironcore_getIdentityInfo(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get the signable data for an invite token.
     * Returns serialized token data (without signature) suitable for Ed25519 signing.
     * The token_bytes must be a bincode-serialized InviteToken.
     * @param {Uint8Array} token_bytes
     * @returns {Uint8Array}
     */
    getInviteSignableData(token_bytes) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(token_bytes, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_getInviteSignableData(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v2 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get the current IronCore operating mode.
     * @returns {any}
     */
    getIronCoreMode() {
        const ret = wasm.ironcore_getIronCoreMode(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get current swarm listeners.
     * @returns {Promise<any>}
     */
    getListeners() {
        const ret = wasm.ironcore_getListeners(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get the NAT status. In browser environments this always returns "unknown".
     * @returns {string}
     */
    getNatStatus() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_getNatStatus(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get the combined overall abuse score for a peer.
     * Combines base reputation and spam confidence into a single score.
     * @param {string} peer_id
     * @returns {number | undefined}
     */
    getOverallScore(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_getOverallScore(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r2 = getDataViewMemory0().getFloat64(retptr + 8 * 1, true);
            return r0 === 0 ? undefined : r2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} peer_id
     * @returns {number}
     */
    getPeerReputation(peer_id) {
        const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_getPeerReputation(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Get currently connected peer IDs.
     * @returns {Promise<any>}
     */
    getPeers() {
        const ret = wasm.ironcore_getPeers(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {any}
     */
    getPrivacyConfig() {
        const ret = wasm.ironcore_getPrivacyConfig(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get the registration state for a given identity.
     * @param {string} identity_id
     * @returns {any}
     */
    getRegistrationState(identity_id) {
        const ptr0 = passStringToWasm0(identity_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_getRegistrationState(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Get the seniority timestamp for this installation.
     * @returns {bigint | undefined}
     */
    getSeniorityTimestamp() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_getSeniorityTimestamp(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r2 = getDataViewMemory0().getBigInt64(retptr + 8 * 1, true);
            return r0 === 0 ? undefined : BigInt.asUintN(64, r2);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Return the current MeshSettings as a JS object.
     * @returns {any}
     */
    getSettings() {
        const ret = wasm.ironcore_getSettings(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get a DSPy signature description by role name.
     * Returns the signature description string if the role is found.
     * @param {string} role
     * @returns {string | undefined}
     */
    getSignature(role) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(role, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_getSignature(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v2;
            if (r0 !== 0) {
                v2 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Import an identity from an encrypted backup string produced by `exportIdentityBackup`.
     * Requires the same passphrase that was used during export.
     * @param {string} backup
     * @param {string} passphrase
     */
    importIdentityBackup(backup, passphrase) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(backup, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(passphrase, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            wasm.ironcore_importIdentityBackup(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {number}
     */
    inboxCount() {
        const ret = wasm.ironcore_inboxCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Initialize identity in local (non-daemon) mode.
     * This generates a new identity or loads an existing one from storage.
     * In Daemon mode, call `initializeIdentityFromDaemon` instead.
     */
    initializeIdentity() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_initializeIdentity(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Initialize identity from daemon when in Daemon mode.
     * This fetches the identity from the local CLI daemon via JSON-RPC.
     * @returns {Promise<void>}
     */
    initializeIdentityFromDaemon() {
        const ret = wasm.ironcore_initializeIdentityFromDaemon(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Check whether a peer is currently blocked.
     * @param {string} peer_id
     * @returns {boolean}
     */
    isPeerBlocked(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_isPeerBlocked(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {boolean}
     */
    isRunning() {
        const ret = wasm.ironcore_isRunning(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * List all blocked peers. Returns a JS array of BlockedIdentity objects.
     * @returns {Array<any>}
     */
    listBlockedPeers() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_listBlockedPeers(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Remove a message from the Rust outbox after it has been delivered.
     * Returns `true` if the message was found and removed, `false` if not found.
     * @param {string} message_id
     * @returns {boolean}
     */
    markMessageSent(message_id) {
        const ptr0 = passStringToWasm0(message_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_markMessageSent(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    constructor() {
        const ret = wasm.ironcore_new();
        this.__wbg_ptr = ret;
        IronCoreFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Notify the core that a peer disconnected from the network.
     * @param {string} peer_id
     */
    notifyPeerDisconnected(peer_id) {
        const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.ironcore_notifyPeerDisconnected(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Notify the core that a peer was discovered on the network.
     * @param {string} peer_id
     */
    notifyPeerDiscovered(peer_id) {
        const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.ironcore_notifyPeerDiscovered(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {number}
     */
    outboxCount() {
        const ret = wasm.ironcore_outboxCount(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Perform storage maintenance (quota enforcement, cleanup).
     */
    performMaintenance() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_performMaintenance(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Generate a cover traffic payload — random bytes that look like an
     * encrypted message. Broadcast via `sendPreparedEnvelope` or the
     * swarm's send-to-all to obscure real traffic patterns.
     * `sizeBytes` is clamped to [16, 1024].
     * @param {number} size_bytes
     * @returns {Uint8Array}
     */
    prepareCoverTraffic(size_bytes) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_prepareCoverTraffic(retptr, this.__wbg_ptr, size_bytes);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Prepare an encrypted message envelope.
     *
     * In **Full mode**: Encrypts locally using the local identity and returns the envelope bytes.
     * In **Daemon mode**: Routes the request to the CLI daemon via JSON-RPC.
     *                 Returns the envelope bytes prepared by the daemon.
     * @param {string} recipient_public_key_hex
     * @param {string} text
     * @returns {Uint8Array}
     */
    prepareMessage(recipient_public_key_hex, text) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(recipient_public_key_hex, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(text, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            wasm.ironcore_prepareMessage(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v3 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v3;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Prepare an encrypted message envelope and return both the message ID
     * and the raw envelope bytes. Use the ID to track delivery receipts.
     * @param {string} recipient_public_key_hex
     * @param {string} text
     * @returns {any}
     */
    prepareMessageWithId(recipient_public_key_hex, text) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(recipient_public_key_hex, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(text, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            wasm.ironcore_prepareMessageWithId(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Prepare a delivery receipt envelope to send back to the original sender.
     * Call this after successfully decoding a received message.
     * @param {string} recipient_public_key_hex
     * @param {string} message_id
     * @returns {Uint8Array}
     */
    prepareReceipt(recipient_public_key_hex, message_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(recipient_public_key_hex, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(message_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            wasm.ironcore_prepareReceipt(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v3 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
            return v3;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Publish data to a gossipsub topic.
     * @param {string} topic
     * @param {Uint8Array} data
     * @returns {Promise<void>}
     */
    publishTopic(topic, data) {
        const ptr0 = passStringToWasm0(topic, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_publishTopic(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return takeObject(ret);
    }
    /**
     * @param {Uint8Array} envelope_bytes
     * @returns {any}
     */
    receiveMessage(envelope_bytes) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(envelope_bytes, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_receiveMessage(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Record a log line into the core log manager.
     * @param {string} line
     */
    recordLog(line) {
        const ptr0 = passStringToWasm0(line, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.ironcore_recordLog(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Resolve any identifier format (peer ID, identity hash, public key hex)
     * to the canonical public_key_hex.
     * @param {string} any_id
     * @returns {string}
     */
    resolveIdentity(any_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(any_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_resolveIdentity(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr2 = r0;
            var len2 = r1;
            if (r3) {
                ptr2 = 0; len2 = 0;
                throw takeObject(r2);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Resolve any identifier format to the identity_id (Blake3 hash).
     * @param {string} any_id
     * @returns {string}
     */
    resolveToIdentityId(any_id) {
        let deferred3_0;
        let deferred3_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(any_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_resolveToIdentityId(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr2 = r0;
            var len2 = r1;
            if (r3) {
                ptr2 = 0; len2 = 0;
                throw takeObject(r2);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Send a prepared encrypted envelope to a connected libp2p peer.
     * @param {string} peer_id
     * @param {Uint8Array} envelope_bytes
     * @returns {Promise<void>}
     */
    sendPreparedEnvelope(peer_id, envelope_bytes) {
        const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(envelope_bytes, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_sendPreparedEnvelope(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return takeObject(ret);
    }
    /**
     * Send data to all currently connected peers.
     * @param {Uint8Array} data
     * @returns {Promise<void>}
     */
    sendToAllPeers(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_sendToAllPeers(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Set the daemon WebSocket socket URL for JSON-RPC communication.
     * This is required when operating in Daemon mode.
     * @param {string} url
     */
    setDaemonSocketUrl(url) {
        const ptr0 = passStringToWasm0(url, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.ironcore_setDaemonSocketUrl(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Set the operating mode for this IronCore instance.
     * In Daemon mode, the WASM client routes all operations through a local CLI daemon
     * via JSON-RPC instead of performing operations locally.
     * @param {any} mode
     */
    setIronCoreMode(mode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_setIronCoreMode(retptr, this.__wbg_ptr, addHeapObject(mode));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the nickname for the local identity.
     * @param {string} nickname
     */
    setNickname(nickname) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(nickname, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_setNickname(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {any} js_config
     */
    setPrivacyConfig(js_config) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_setPrivacyConfig(retptr, this.__wbg_ptr, addHeapObject(js_config));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {Uint8Array} data
     * @returns {any}
     */
    signData(data) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_signData(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    start() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_start(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * DEPRECATED shim for pre-0.1.2 clients.
     *
     * This maps a relay URL to a libp2p websocket multiaddr and starts the
     * wasm swarm path.  Unlike the original fire-and-forget API, this version
     * is `async` and propagates start errors back to the JS caller so callers
     * can detect and handle failures rather than silently missing them.
     *
     * Prefer `startSwarm(bootstrapAddrs)` for new code.
     * @param {string} relay_url
     * @returns {Promise<void>}
     */
    startReceiveLoop(relay_url) {
        const ptr0 = passStringToWasm0(relay_url, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_startReceiveLoop(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Start libp2p swarm networking for the browser client.
     *
     * `bootstrapAddrs` must be a JS array of libp2p multiaddr strings.
     * @param {any} bootstrap_addrs
     * @returns {Promise<void>}
     */
    startSwarm(bootstrap_addrs) {
        const ret = wasm.ironcore_startSwarm(this.__wbg_ptr, addHeapObject(bootstrap_addrs));
        return takeObject(ret);
    }
    stop() {
        wasm.ironcore_stop(this.__wbg_ptr);
    }
    /**
     * Stop libp2p swarm networking for the browser client.
     * @returns {Promise<void>}
     */
    stopSwarm() {
        const ret = wasm.ironcore_stopSwarm(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Subscribe to a gossipsub topic.
     * @param {string} topic
     * @returns {Promise<void>}
     */
    subscribeTopic(topic) {
        const ptr0 = passStringToWasm0(topic, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_subscribeTopic(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Touch (update last-seen timestamp for) a notification endpoint.
     * @param {string} endpoint_id
     */
    touchNotificationEndpoint(endpoint_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(endpoint_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_touchNotificationEndpoint(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Unblock a previously blocked peer.
     * @param {string} peer_id
     */
    unblockPeer(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_unblockPeer(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Unregister a notification endpoint by ID.
     * @param {string} endpoint_id
     */
    unregisterNotificationEndpoint(endpoint_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(endpoint_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.ironcore_unregisterNotificationEndpoint(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Unsubscribe from a gossipsub topic.
     * @param {string} topic
     * @returns {Promise<void>}
     */
    unsubscribeTopic(topic) {
        const ptr0 = passStringToWasm0(topic, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_unsubscribeTopic(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Update disk usage statistics for quota enforcement.
     * @param {bigint} total_bytes
     * @param {bigint} free_bytes
     */
    updateDiskStats(total_bytes, free_bytes) {
        wasm.ironcore_updateDiskStats(this.__wbg_ptr, total_bytes, free_bytes);
    }
    /**
     * Apply a partial or full settings update from JS.
     * Accepts a JS object matching the WasmMeshSettings shape.
     * @param {any} js_settings
     */
    updateSettings(js_settings) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_updateSettings(retptr, this.__wbg_ptr, addHeapObject(js_settings));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Validate the given settings against invariant rules.
     * @param {any} js_settings
     */
    validateSettings(js_settings) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ironcore_validateSettings(retptr, this.__wbg_ptr, addHeapObject(js_settings));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {Uint8Array} data
     * @param {Uint8Array} signature
     * @param {string} public_key_hex
     * @returns {boolean}
     */
    verifySignature(data, signature, public_key_hex) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passArray8ToWasm0(signature, wasm.__wbindgen_export);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(public_key_hex, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len2 = WASM_VECTOR_LEN;
            wasm.ironcore_verifySignature(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} storage_path
     * @returns {IronCore}
     */
    static withStorage(storage_path) {
        const ptr0 = passStringToWasm0(storage_path, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_withStorage(ptr0, len0);
        return IronCore.__wrap(ret);
    }
    /**
     * @param {string} storage_path
     * @returns {Promise<IronCore>}
     */
    static withStorageAsync(storage_path) {
        const ptr0 = passStringToWasm0(storage_path, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ironcore_withStorageAsync(ptr0, len0);
        return takeObject(ret);
    }
}
if (Symbol.dispose) IronCore.prototype[Symbol.dispose] = IronCore.prototype.free;

/**
 * Notification manager for WASM browser environment
 */
export class NotificationManager {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        NotificationManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_notificationmanager_free(ptr, 0);
    }
    /**
     * Close all notifications (placeholder for tracking)
     */
    closeAllNotifications() {
        wasm.notificationmanager_closeAllNotifications(this.__wbg_ptr);
    }
    /**
     * Detect the current browser type
     * @returns {string}
     */
    detectBrowser() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.notificationmanager_detectBrowser(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get browser-specific notification options as JSON
     * @returns {any}
     */
    getBrowserOptions() {
        const ret = wasm.notificationmanager_getBrowserOptions(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get the current notification permission
     * @returns {string}
     */
    getPermission() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.notificationmanager_getPermission(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Check if notification permission is granted
     * @returns {boolean}
     */
    isPermissionGranted() {
        const ret = wasm.notificationmanager_isPermissionGranted(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Check if notifications are supported in the current browser
     * @returns {boolean}
     */
    isSupported() {
        const ret = wasm.notificationmanager_isSupported(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Create a new notification manager
     */
    constructor() {
        const ret = wasm.notificationmanager_new();
        this.__wbg_ptr = ret;
        NotificationManagerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Request notification permission from the user
     * @returns {Promise<boolean>}
     */
    requestPermission() {
        const ret = wasm.notificationmanager_requestPermission(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Show a notification with the given options
     * @param {string} title
     * @param {string} body
     * @param {any} _options
     * @returns {Promise<void>}
     */
    showNotification(title, body, _options) {
        const ptr0 = passStringToWasm0(title, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(body, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.notificationmanager_showNotification(this.__wbg_ptr, ptr0, len0, ptr1, len1, addHeapObject(_options));
        return takeObject(ret);
    }
    /**
     * Show permission guidance UI when permission is denied
     */
    showPermissionGuidance() {
        wasm.notificationmanager_showPermissionGuidance(this.__wbg_ptr);
    }
}
if (Symbol.dispose) NotificationManager.prototype[Symbol.dispose] = NotificationManager.prototype.free;

export class WasmContactManager {
    static __wrap(ptr) {
        const obj = Object.create(WasmContactManager.prototype);
        obj.__wbg_ptr = ptr;
        WasmContactManagerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmContactManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcontactmanager_free(ptr, 0);
    }
    /**
     * @param {any} js_contact
     */
    add(js_contact) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcontactmanager_add(retptr, this.__wbg_ptr, addHeapObject(js_contact));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {number}
     */
    count() {
        const ret = wasm.wasmcontactmanager_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Flush contact data to persistent storage.
     */
    flush() {
        wasm.wasmcontactmanager_flush(this.__wbg_ptr);
    }
    /**
     * @param {string} peer_id
     * @returns {any}
     */
    get(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_get(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {Array<any>}
     */
    list() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcontactmanager_list(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} peer_id
     */
    remove(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_remove(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Search contacts by query string (matches peer_id, nickname, notes).
     * @param {string} query
     * @returns {Array<any>}
     */
    search(query) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_search(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} peer_id
     * @param {string | null} [nickname]
     */
    setLocalNickname(peer_id, nickname) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(nickname) ? 0 : passStringToWasm0(nickname, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_setLocalNickname(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set the federated (broadcast) nickname for a contact.
     * @param {string} peer_id
     * @param {string | null} [nickname]
     */
    setNickname(peer_id, nickname) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(nickname) ? 0 : passStringToWasm0(nickname, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_setNickname(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Update the last known device ID for a contact.
     * @param {string} peer_id
     * @param {string | null} [device_id]
     */
    updateDeviceId(peer_id, device_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            var ptr1 = isLikeNone(device_id) ? 0 : passStringToWasm0(device_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_updateDeviceId(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Update the last-seen timestamp for a contact.
     * @param {string} peer_id
     */
    updateLastSeen(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmcontactmanager_updateLastSeen(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) WasmContactManager.prototype[Symbol.dispose] = WasmContactManager.prototype.free;

export class WasmHistoryManager {
    static __wrap(ptr) {
        const obj = Object.create(WasmHistoryManager.prototype);
        obj.__wbg_ptr = ptr;
        WasmHistoryManagerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmHistoryManagerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmhistorymanager_free(ptr, 0);
    }
    /**
     * @param {any} js_record
     */
    add(js_record) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhistorymanager_add(retptr, this.__wbg_ptr, addHeapObject(js_record));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    clear() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhistorymanager_clear(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Clear all messages in a conversation with a specific peer.
     * Alias: this is equivalent to `removeConversation` in the core API.
     * @param {string} peer_id
     */
    clearConversation(peer_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_clearConversation(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} peer_id
     * @param {number} limit
     * @returns {Array<any>}
     */
    conversation(peer_id, limit) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(peer_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_conversation(retptr, this.__wbg_ptr, ptr0, len0, limit);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {number}
     */
    count() {
        const ret = wasm.wasmhistorymanager_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Delete a single message by ID.
     * @param {string} id
     */
    delete(id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_delete(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {number} max_messages
     * @returns {number}
     */
    enforceRetention(max_messages) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhistorymanager_enforceRetention(retptr, this.__wbg_ptr, max_messages);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Flush history data to persistent storage.
     */
    flush() {
        wasm.wasmhistorymanager_flush(this.__wbg_ptr);
    }
    /**
     * Get a single message record by ID.
     * @param {string} id
     * @returns {any}
     */
    get(id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_get(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Mark a message as delivered by ID.
     * @param {string} id
     */
    markDelivered(id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_markDelivered(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            if (r1) {
                throw takeObject(r0);
            }
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {bigint} before_timestamp
     * @returns {number}
     */
    pruneBefore(before_timestamp) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhistorymanager_pruneBefore(retptr, this.__wbg_ptr, before_timestamp);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 >>> 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string | null | undefined} peer_filter
     * @param {number} limit
     * @returns {Array<any>}
     */
    recent(peer_filter, limit) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            var ptr0 = isLikeNone(peer_filter) ? 0 : passStringToWasm0(peer_filter, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_recent(retptr, this.__wbg_ptr, ptr0, len0, limit);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Search message history by query string.
     * @param {string} query
     * @param {number} limit
     * @returns {Array<any>}
     */
    search(query, limit) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmhistorymanager_search(retptr, this.__wbg_ptr, ptr0, len0, limit);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {any}
     */
    stats() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmhistorymanager_stats(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) WasmHistoryManager.prototype[Symbol.dispose] = WasmHistoryManager.prototype.free;

export function init_logging() {
    wasm.init_logging();
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_bce6d499ff0a4aff: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_Number_b7972a139bfbfdf0: function(arg0) {
            const ret = Number(getObject(arg0));
            return ret;
        },
        __wbg_String_8564e559799eccda: function(arg0, arg1) {
            const ret = String(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_WorkerGlobalScope_0f395e12ab05d182: function(arg0) {
            const ret = getObject(arg0).WorkerGlobalScope;
            return addHeapObject(ret);
        },
        __wbg___wbindgen_bigint_get_as_i64_410e28c7b761ad83: function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_boolean_get_2304fb8c853028c8: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_edece8177ad01481: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_07056af4f902c445: function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_is_bigint_aeae3893f30ed54e: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        },
        __wbg___wbindgen_is_function_5cd60d5cf78b4eef: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_2042690d351e14f0: function(arg0) {
            const ret = getObject(arg0) === null;
            return ret;
        },
        __wbg___wbindgen_is_object_b4593df85baada48: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_dde0fd9020db4434: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_35bb9f4c7fd651d5: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_eq_c0ed08b3e0f393b9: function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_0ad77b7717db155c: function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_number_get_f73a1244370fcc2c: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_d109740c0d18f4d7: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_9c31b086c2b26051: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_3fa391f3fcdb55f8: function(arg0) {
            getObject(arg0)._wbg_cb_unref();
        },
        __wbg_appendChild_6e88800a9a8fb360: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).appendChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_apply_345835e57390767e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.apply(getObject(arg0), getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_body_2ac005c657a3d103: function(arg0) {
            const ret = getObject(arg0).body;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_bufferedAmount_4451d2084a78a8a1: function(arg0) {
            const ret = getObject(arg0).bufferedAmount;
            return ret;
        },
        __wbg_call_13665d9f14390edc: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_call_dfde26266607c996: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_clearInterval_d8d28e6a4cc7f7cd: function(arg0, arg1) {
            getObject(arg0).clearInterval(arg1);
        },
        __wbg_clearInterval_edc7095bc6e9c57a: function(arg0, arg1) {
            getObject(arg0).clearInterval(arg1);
        },
        __wbg_clearTimeout_01406e55473040f6: function(arg0) {
            const ret = clearTimeout(takeObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_close_e1f28ed6c57d2b8b: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            getObject(arg0).close(arg1, getStringFromWasm0(arg2, arg3));
        }, arguments); },
        __wbg_construct_391d7e1e0b4dda3a: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.construct(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_crypto_38df2bab126b63dc: function(arg0) {
            const ret = getObject(arg0).crypto;
            return addHeapObject(ret);
        },
        __wbg_data_5fc79a19e47d1531: function(arg0) {
            const ret = getObject(arg0).data;
            return addHeapObject(ret);
        },
        __wbg_document_3540635616a18455: function(arg0) {
            const ret = getObject(arg0).document;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_done_54b8da57023b7ed2: function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        },
        __wbg_entries_564a7e8b1e54ede5: function(arg0) {
            const ret = Object.entries(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_focus_964e41d2e667c228: function() { return handleError(function (arg0) {
            getObject(arg0).focus();
        }, arguments); },
        __wbg_getItem_88cc26174f98c20c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg1).getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_getRandomValues_3f44b700395062e5: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getRandomValues_c44a50d8cfdaebeb: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).getRandomValues(getObject(arg1));
        }, arguments); },
        __wbg_getRandomValues_ef12552bf5acd2fe: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_get_3e9a707ab7d352eb: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_get_98fdf51d029a75eb: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_dcf82ab8aad1a593: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_get_unchecked_1dfe6d05ad91d9b7: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        },
        __wbg_instanceof_ArrayBuffer_53db37b06f6b9afe: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_abd07d4bd221d50b: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_faa5cf994f49cca7: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_ironcore_new: function(arg0) {
            const ret = IronCore.__wrap(arg0);
            return addHeapObject(ret);
        },
        __wbg_isArray_94898ed3aad6947b: function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        },
        __wbg_isSafeInteger_01e964d144ad3a55: function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        },
        __wbg_iterator_1441b47f341dc34f: function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        },
        __wbg_length_2591a0f4f659a55c: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_56fcd3e2b7e0299d: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_log_0c201ade58bb55e1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), getStringFromWasm0(arg4, arg5), getStringFromWasm0(arg6, arg7));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_log_ce2c4456b290c5e7: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.log(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_log_eb752234eec406d1: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_mark_b4d943f3bc2d2404: function(arg0, arg1) {
            performance.mark(getStringFromWasm0(arg0, arg1));
        },
        __wbg_measure_84362959e621a2c1: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            let deferred0_0;
            let deferred0_1;
            let deferred1_0;
            let deferred1_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                deferred1_0 = arg2;
                deferred1_1 = arg3;
                performance.measure(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
                wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
            }
        }, arguments); },
        __wbg_msCrypto_bd5a034af96bcba6: function(arg0) {
            const ret = getObject(arg0).msCrypto;
            return addHeapObject(ret);
        },
        __wbg_new_02d162bc6cf02f60: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_310879b66b6e95e1: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_7ddec6de44ff8f5d: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_b1280f836646084c: function() { return handleError(function (arg0, arg1) {
            const ret = new WebSocket(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_new_typed_c072c4ce9a2a0cdf: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return __wasm_bindgen_func_elem_6351(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return addHeapObject(ret);
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_with_length_99887c91eae4abab: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_next_2a4e19f4f5083b0f: function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        },
        __wbg_next_6429a146bf756f93: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_node_84ea875411254db1: function(arg0) {
            const ret = getObject(arg0).node;
            return addHeapObject(ret);
        },
        __wbg_now_81363d44c96dd239: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_now_e7c6795a7f81e10f: function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        },
        __wbg_parentNode_1bd06e132b3671e5: function(arg0) {
            const ret = getObject(arg0).parentNode;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_performance_3fcf6e32a7e1ed0a: function(arg0) {
            const ret = getObject(arg0).performance;
            return addHeapObject(ret);
        },
        __wbg_process_44c7a14e11e9f69e: function(arg0) {
            const ret = getObject(arg0).process;
            return addHeapObject(ret);
        },
        __wbg_prototypesetcall_5f9bdc8d75e07276: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_push_b77c476b01548d0a: function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        },
        __wbg_querySelector_b2e645d3370974cb: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        }, arguments); },
        __wbg_queueMicrotask_78d584b53af520f5: function(arg0) {
            const ret = getObject(arg0).queueMicrotask;
            return addHeapObject(ret);
        },
        __wbg_queueMicrotask_b39ea83c7f01971a: function(arg0) {
            queueMicrotask(getObject(arg0));
        },
        __wbg_randomFillSync_6c25eac9869eb53c: function() { return handleError(function (arg0, arg1) {
            getObject(arg0).randomFillSync(takeObject(arg1));
        }, arguments); },
        __wbg_readyState_a1a00cc8898812ac: function(arg0) {
            const ret = getObject(arg0).readyState;
            return ret;
        },
        __wbg_removeChild_d0ea60c90e6c7715: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).removeChild(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_require_b4edbdcf3e2a1ef0: function() { return handleError(function () {
            const ret = module.require;
            return addHeapObject(ret);
        }, arguments); },
        __wbg_resolve_d17db9352f5a220e: function(arg0) {
            const ret = Promise.resolve(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_send_b9d998a91cbc8429: function() { return handleError(function (arg0, arg1, arg2) {
            getObject(arg0).send(getArrayU8FromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_setAttribute_2e611c7b4151e535: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setInterval_27ff30bc4454e68a: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).setInterval(getObject(arg1), arg2, ...getObject(arg3));
            return ret;
        }, arguments); },
        __wbg_setInterval_8ba08205bd6bc93c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = getObject(arg0).setInterval(getObject(arg1), arg2, ...getObject(arg3));
            return ret;
        }, arguments); },
        __wbg_setItem_caab843cd6845dbb: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            getObject(arg0).setItem(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setTimeout_613a21b62dc655a1: function() { return handleError(function (arg0, arg1) {
            const ret = setTimeout(getObject(arg0), arg1);
            return addHeapObject(ret);
        }, arguments); },
        __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        },
        __wbg_set_78ea6a19f4818587: function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        },
        __wbg_set_a0e911be3da02782: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments); },
        __wbg_set_binaryType_5c0002dfcf194934: function(arg0, arg1) {
            getObject(arg0).binaryType = __wbindgen_enum_BinaryType[arg1];
        },
        __wbg_set_onclick_807ae6c28b07f4ad: function(arg0, arg1) {
            getObject(arg0).onclick = getObject(arg1);
        },
        __wbg_set_onclose_3121e15055418a37: function(arg0, arg1) {
            getObject(arg0).onclose = getObject(arg1);
        },
        __wbg_set_onerror_1dd52df6279c8dd2: function(arg0, arg1) {
            getObject(arg0).onerror = getObject(arg1);
        },
        __wbg_set_onmessage_9eb2cf76e70783ad: function(arg0, arg1) {
            getObject(arg0).onmessage = getObject(arg1);
        },
        __wbg_set_onopen_6f3fc5e2ad3144f1: function(arg0, arg1) {
            getObject(arg0).onopen = getObject(arg1);
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_THIS_02344c9b09eb08a9: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_ac6d4ac874d5cd54: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_9b2406c23aeb2023: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_b34d2126934e16ba: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_subarray_7c6a0da8f3b4a1ba: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
            return addHeapObject(ret);
        },
        __wbg_then_837494e384b37459: function(arg0, arg1) {
            const ret = getObject(arg0).then(getObject(arg1));
            return addHeapObject(ret);
        },
        __wbg_then_bd927500e8905df2: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_value_9cc0518af87a489c: function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        },
        __wbg_versions_276b2795b1c6a219: function(arg0) {
            const ret = getObject(arg0).versions;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 2144, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_6334);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("CloseEvent")], shim_idx: 1198, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_3417);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1198, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_3417_2);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MessageEvent")], shim_idx: 1198, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_3417_3);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 1877, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_5077);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000006: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000009: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./scmessenger_wasm_bg.js": import0,
    };
}

function __wasm_bindgen_func_elem_5077(arg0, arg1) {
    wasm.__wasm_bindgen_func_elem_5077(arg0, arg1);
}

function __wasm_bindgen_func_elem_3417(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_3417(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_3417_2(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_3417_2(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_3417_3(arg0, arg1, arg2) {
    wasm.__wasm_bindgen_func_elem_3417_3(arg0, arg1, addHeapObject(arg2));
}

function __wasm_bindgen_func_elem_6334(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_6334(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_6351(arg0, arg1, arg2, arg3) {
    wasm.__wasm_bindgen_func_elem_6351(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}


const __wbindgen_enum_BinaryType = ["blob", "arraybuffer"];
const IronCoreFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ironcore_free(ptr, 1));
const NotificationManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_notificationmanager_free(ptr, 1));
const WasmContactManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcontactmanager_free(ptr, 1));
const WasmHistoryManagerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmhistorymanager_free(ptr, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_export5(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_export5(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('scmessenger_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
