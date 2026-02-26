import SwiftUI

struct DiagnosticsView: View {
    @Environment(MeshRepository.self) private var repository
    @State private var logText: String = ""
    @State private var showingExportSheet = false
    @State private var exportURL: URL?

    var body: some View {
        VStack(spacing: 0) {
            // Technical info header
            VStack(alignment: .leading, spacing: 4) {
                Text("Log File: \(repository.diagnosticsLogPath())")
                Text("File Size: \(logFileSize)")
            }
            .font(.system(size: 10, design: .monospaced))
            .foregroundStyle(.secondary)
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(Color(uiColor: .systemGroupedBackground))

            ScrollView {
                if logText.isEmpty {
                    VStack(spacing: 20) {
                        Image(systemName: "doc.text.magnifyingglass")
                            .font(.largeTitle)
                            .foregroundStyle(.secondary)
                        Text("No Diagnostic Logs Found")
                            .font(.headline)
                        Text("Click 'Manual Trace' or start the Mesh Service to generate logs.")
                            .font(.subheadline)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.top, 100)
                    .foregroundStyle(.secondary)
                } else {
                    Text(logText)
                        .font(.system(.caption, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding()
                }
            }
            .background(Color(uiColor: .secondarySystemBackground))
            
            Divider()
            
            VStack(spacing: 12) {
                HStack(spacing: 12) {
                    Button {
                        refreshLogs()
                    } label: {
                        Label("Refresh", systemImage: "arrow.clockwise")
                    }
                    .buttonStyle(.bordered)

                    Button {
                        addManualTrace()
                    } label: {
                        Label("Trace", systemImage: "bolt.fill")
                    }
                    .buttonStyle(.bordered)

                    Button(role: .destructive) {
                        clearLogs()
                    } label: {
                        Label("Clear", systemImage: "trash")
                    }
                    .buttonStyle(.bordered)
                }

                Button {
                    prepareExport()
                } label: {
                    Label("Export Logistics Bundle", systemImage: "square.and.arrow.up")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
            }
            .padding()
        }
        .navigationTitle("Diagnostics")
        .navigationBarTitleDisplayMode(.inline)
        .onAppear {
            refreshLogs()
        }
        .sheet(isPresented: $showingExportSheet) {
            if let url = exportURL {
                ShareSheet(items: [url])
            }
        }
    }

    private func refreshLogs() {
        logText = repository.diagnosticsSnapshot(limit: 500)
    }

    private func addManualTrace() {
        repository.appendDiagnostic("manual_trace_requested")
        refreshLogs()
    }

    private func clearLogs() {
        repository.clearDiagnostics()
        refreshLogs()
    }

    private func prepareExport() {
        let path = repository.diagnosticsLogPath()
        let url = URL(fileURLWithPath: path)
        exportURL = url
        showingExportSheet = true
    }

    private var logFileSize: String {
        let path = repository.diagnosticsLogPath()
        guard let attrs = try? FileManager.default.attributesOfItem(atPath: path),
              let size = attrs[.size] as? Int64 else {
            return "0 B"
        }
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: size)
    }
}

struct ShareSheet: UIViewControllerRepresentable {
    let items: [Any]
    
    func makeUIViewController(context: Context) -> UIActivityViewController {
        UIActivityViewController(activityItems: items, applicationActivities: nil)
    }
    
    func updateUIViewController(_ uiViewController: UIActivityViewController, context: Context) {}
}
