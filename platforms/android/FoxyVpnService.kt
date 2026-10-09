package com.vauth.foxyvpn.vpn

import android.net.VpnService
import android.os.ParcelFileDescriptor

class FoxyVpnService : VpnService() {
    private var tunFd: ParcelFileDescriptor? = null

    private fun establishTunInterface(): ParcelFileDescriptor? {
        return Builder()
            .setSession("FoxyVPN Master")
            .addAddress("10.8.0.2", 32)
            .addRoute("0.0.0.0", 0)
            .addAddress("fd00:1::1", 128)
            .addRoute("::", 0)
            .addDnsServer("1.1.1.1")
            .setMtu(8500)
            .setBlocking(true)
            .apply {
                addDisallowedApplication(packageName)
            }
            .establish()
    }
}
