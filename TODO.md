# Vortex TODO List

## Phase 5: Directory-to-VM Mapping - Completed

### Completed
- ✅ CLI consolidation (removed vortex_sessions, vortex_orchestrator, vortex_discovery)
- ✅ Dead code cleanup
- ✅ Auto-discovery module created (`src/discovery/`)
- ✅ Interactive workspace initialization
- ✅ Non-interactive auto-scan mode
- ✅ YAML configuration file generation with all detected services
- ✅ Backend selection feature (krunvm/firecracker support)
- ✅ Graceful degradation when no backend installed
- ✅ Updated documentation

### Notes
Phase 5 delivers complete workspace initialization with auto-discovery, language detection, service inference, and backend configuration. The system works correctly without backends for configuration generation, and fails gracefully when VM operations require a backend.

### Remaining
- [ ] `vortex service list` command to list detected services
- [ ] `vortex workspace create --from-dir` integration
- [ ] DevContainer import command (`vortex workspace import`)
- [ ] `vortex run` backend configuration support

---

## Phase 6: Development Context System

### Tasks
- [ ] Context definitions (dev/staging/prod)
- [ ] Context switching (`vortex context switch dev`)
- [ ] Development tools (auto-inject debug ports, hot reload)
- [ ] Resource scaling per context
- [ ] Context-specific configurations

---

## Phase 7: Real File Synchronization

### Tasks
- [ ] Live file watching (inotify/fsevents)
- [ ] Bidirectional sync (host ↔ VM)
- [ ] Conflict resolution for simultaneous edits
- [ ] Selective sync (`.vortexignore`)
- [ ] Performance optimization (batch updates, rsync-like)

---

## Phase 8: Editor Integration

### Tasks
- [ ] VS Code Remote integration
- [ ] Development tunnels
- [ ] Editor plugins (Vortex workspace awareness)
- [ ] Multiple editor support (Neovim, IntelliJ)

---

## Phase 9: Advanced Orchestration

### Tasks
- [ ] Service dependencies (start order, health checks)
- [ ] Load balancing (multiple instances)
- [ ] Service discovery (DNS, environment variables)
- [ ] Distributed logging (centralized aggregation)
- [ ] Metrics collection (Prometheus integration)

---

## Phase 10: Cloud & Collaboration

### Tasks
- [ ] Remote VMs (AWS, GCP, Azure)
- [ ] Team workspaces (shared environments)
- [ ] Environment templates (marketplace)
- [ ] CI/CD integration (deploy from git)
- [ ] Resource management (auto-scaling, cost optimization)

---

## Immediate Next Steps

### High Priority
1. **Complete Phase 5** - Finish the discovery engine integration
2. **Unit Tests** - Add unit tests for core library modules
3. **Performance Testing** - Validate speed claims against Docker

### Medium Priority
4. **Documentation** - Add more examples and tutorials
5. **Error Handling** - Improve error messages for common failures
6. **Testing** - Add integration tests for workspace initialization

### Low Priority
7. **Windows Support** - Add Windows platform support
8. **Plugins** - Design plugin system architecture
9. **Cloud** - Research cloud provider integrations

---

## Bugs to Fix

- [ ] None currently identified

---

*Last updated: 2026-02-11*
