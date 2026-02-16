//
//  MeshDashboardView.swift
//  SCMessenger
//
//  Mesh network dashboard
//

import SwiftUI

struct MeshDashboardView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var stats: ServiceStats?
    
    var body: some View {
        ScrollView {
            VStack(spacing: Theme.spacingLarge) {
                ServiceStatusCard(stats: stats)
                
                TransportStatusSection()
                
                if let stats = stats {
                    RelayStatsSection(stats: stats)
                }
            }
            .padding(Theme.spacingMedium)
        }
        .navigationTitle("Mesh Dashboard")
        .task {
            loadDashboardData()
        }
    }
    
    private func loadDashboardData() {
        repository.updateStats()
        stats = repository.serviceStats
    }
}

struct ServiceStatusCard: View {
    let stats: ServiceStats?
    
    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            HStack {
                Image(systemName: "network")
                    .font(.title2)
                Text("Service Status")
                    .font(Theme.titleLarge)
                Spacer()
                Circle()
                    .fill(stats != nil ? Color.green : Color.gray)
                    .frame(width: 12, height: 12)
            }
            
            if let stats = stats {
                Divider()
                
                StatRow(label: "Peers Discovered", value: "\(stats.peersDiscovered)")
                StatRow(label: "Messages Relayed", value: "\(stats.messagesRelayed)")
                StatRow(label: "Bytes Transferred", value: formatBytes(stats.bytesTransferred))
                StatRow(label: "Uptime", value: formatUptime(stats.uptimeSecs))
            } else {
                Text("Service not running")
                    .font(Theme.bodyMedium)
                    .foregroundStyle(Theme.onSurfaceVariant)
            }
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }
    
    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }
    
    private func formatUptime(_ seconds: UInt64) -> String {
        let hours = seconds / 3600
        let minutes = (seconds % 3600) / 60
        return "\(hours)h \(minutes)m"
    }
}

struct StatRow: View {
    let label: String
    let value: String
    
    var body: some View {
        HStack {
            Text(label)
                .font(Theme.bodyMedium)
            Spacer()
            Text(value)
                .font(Theme.titleMedium)
                .foregroundStyle(Theme.onPrimaryContainer)
        }
    }
}

struct TransportStatusSection: View {
    @Environment(MeshRepository.self) private var repository

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            Text("Transports")
                .font(Theme.titleLarge)
            
            TransportStatusRow(type: .multipeer, isActive: true)
            TransportStatusRow(type: .ble, isActive: true)
            TransportStatusRow(type: .internet, isActive: repository.networkStatus.available)
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }
}

struct TransportStatusRow: View {
    let type: TransportType
    let isActive: Bool
    
    var body: some View {
        HStack(spacing: Theme.spacingMedium) {
            Image(systemName: type.icon)
                .font(.title3)
                .foregroundStyle(isActive ? Theme.onSuccessContainer : Theme.onSurfaceVariant)
                .frame(width: 30)
            
            Text(type.rawValue)
                .font(Theme.bodyMedium)
            
            Spacer()
            
            Circle()
                .fill(isActive ? Theme.onSuccessContainer : Color.gray)
                .frame(width: 8, height: 8)
        }
    }
}

struct RelayStatsSection: View {
    let stats: ServiceStats
    
    var body: some View {
        VStack(alignment: .leading, spacing: Theme.spacingMedium) {
            HStack {
                Image(systemName: "arrow.triangle.2.circlepath")
                    .font(.title2)
                Text("Relay Stats")
                    .font(Theme.titleLarge)
            }
            
            Divider()
            
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Messages Relayed")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                    Text("\(stats.messagesRelayed)")
                        .font(Theme.headlineMedium)
                }
                
                Spacer()
                
                VStack(alignment: .trailing, spacing: 4) {
                    Text("Bytes Transferred")
                        .font(Theme.bodySmall)
                        .foregroundStyle(Theme.onSurfaceVariant)
                    Text(formatBytes(stats.bytesTransferred))
                        .font(Theme.headlineMedium)
                }
            }
        }
        .padding(Theme.spacingMedium)
        .themedCard()
    }
    
    private func formatBytes(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }
}
