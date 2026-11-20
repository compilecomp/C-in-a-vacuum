# CIV C IN A VACUUM
# COMPLETE SYSTEM ENGINEERING SPECIFICATION

**Document ID:** CIV-MASTER-001  
**Revision:** 1.1  
**Architecture:** x86-64  
**Kernel:** Monolithic  
**GPU interface:** Vulkan  
**Verification scope:** End-to-end  
**Primary implementation language:** Rust  
**Low-level implementation:** x86-64 assembly  
**Formal specification:** Lean 4, with complementary verification systems  
**Primary objectives:** Maximum performance, formal correctness, security, reliability

---

# 0. PROJECT DEFINITION

## 0.1 What CIV is

CIV is a general-purpose operating system.

It is not merely:
- A kernel.
- A microkernel.
- A hypervisor.
- A verified memory manager.
- A collection of verified drivers.
- A formally verified program.

It is an integrated computing system comprising:

1. Hardware specifications.
2. Firmware interfaces.
3. Boot infrastructure.
4. Kernel.
5. Kernel libraries.
6. Memory-management infrastructure.
7. Process and execution infrastructure.
8. Device drivers.
9. Graphics and compute infrastructure.
10. Storage infrastructure.
11. Networking infrastructure.
12. Filesystems.
13. Security infrastructure.
14. Cryptographic infrastructure.
15. User-space services.
16. Runtime libraries.
17. Application interfaces.
18. Development toolchain.
19. Formal verification toolchain.
20. Testing infrastructure.
21. Build infrastructure.
22. Deployment infrastructure.
23. Recovery infrastructure.
24. Physical security infrastructure.

Every component MUST have a specification, an implementation owner, defined dependencies, and an acceptance criterion.

## 0.2 Primary requirements

CIV-REQ-001: Preserve system correctness.

CIV-REQ-002: Preserve memory safety.

CIV-REQ-003: Preserve authority boundaries.

CIV-REQ-004: Preserve concurrency invariants.

CIV-REQ-005: Minimize execution overhead.

CIV-REQ-006: Minimize memory overhead.

CIV-REQ-007: Minimize unnecessary data movement.

CIV-REQ-008: Minimize unnecessary privilege transitions.

CIV-REQ-009: Prevent unauthorized information disclosure within the declared threat model.

CIV-REQ-010: Account for microarchitectural leakage.

CIV-REQ-011: Account for electromagnetic leakage.

CIV-REQ-012: Account for hardware faults and malicious devices.

CIV-REQ-013: Provide reproducible build artifacts.

CIV-REQ-014: Maintain end-to-end verification evidence.

CIV-REQ-015: Make every performance claim measurable.

## 0.3 System completeness rule

A subsystem is not considered specified merely because its name appears in this document.

Every subsystem MUST define:

- Purpose.
- Functional requirements.
- Internal components.
- Algorithms.
- Data structures.
- Interfaces.
- Input formats.
- Output formats.
- Ownership rules.
- Resource requirements.
- Hardware dependencies.
- Security requirements.
- Failure behavior.
- Concurrency behavior.
- Performance requirements.
- Formal properties.
- Verification strategy.
- Implementation language.
- Test strategy.
- Acceptance criteria.

Any unspecified field MUST be marked `UNDEFINED`, not silently assumed.

---

# 1. HARDWARE ENGINEERING

CIV requires a complete hardware compatibility and trust specification.

The hardware is not simply something the kernel happens to run on. It is part of the system's behavior and security boundary.

## 1.1 CPU platform

### Components to define

1. CPU architectural profile.
2. CPUID discovery subsystem.
3. CPU feature registry.
4. Model-specific register interface.
5. Control-register management.
6. General-purpose register context.
7. SIMD register context.
8. Floating-point state management.
9. XSAVE state management.
10. FPU initialization.
11. APIC initialization.
12. Local APIC management.
13. I/O APIC management.
14. Interrupt descriptor table.
15. Global descriptor table.
16. Task-state segment configuration.
17. System-call entry and exit.
18. CPU exception handling.
19. Machine-check handling.
20. CPU hotplug.
21. CPU topology discovery.
22. CPU affinity management.
23. CPU power management.
24. Microcode management.
25. CPU performance-counter management.
26. Hardware random-number interface.
27. Speculation mitigation management.
28. CPU errata database.

### Algorithms and mechanisms

- CPU enumeration.
- Feature intersection across cores.
- Per-core initialization.
- Context switching.
- Extended-state save and restore.
- Interrupt dispatch.
- Exception dispatch.
- CPU-local data access.
- CPU migration.
- AP startup.
- TLB shootdown coordination.
- CPU rendezvous.
- Power-state transitions.

### Required contracts

- Architectural state preservation.
- Privilege separation.
- Correct interrupt entry.
- Correct exception return.
- Defined memory ordering.
- Defined atomic operations.
- Correct handling of heterogeneous feature availability.
- Correct handling of CPU removal.

### Hardware profiles

The project MUST define supported Intel and AMD processor families separately.

It MUST NOT assume that every x86-64 processor implements the same optional extensions or has identical microarchitectural behavior.

---

## 1.2 Motherboard and platform controller

### Components

- Platform discovery.
- ACPI parser.
- ACPI table validator.
- PCI configuration interface.
- PCI enumeration.
- PCI resource allocator.
- PCI bridge management.
- PCIe capability discovery.
- Interrupt routing.
- MSI/MSI-X configuration.
- Firmware configuration reader.
- System management interface.
- Watchdog interface.
- Power controller interface.
- Hardware sensor interface.
- Reset controller.
- Clock source discovery.

### Required definitions

- Resource allocation.
- MMIO ownership.
- Bus enumeration.
- Device identity.
- Interrupt ownership.
- Hotplug semantics.
- Firmware-provided data validation.
- Hardware error reporting.

---

## 1.3 RAM and memory hardware

### Components

- Physical memory discovery.
- Memory-region registry.
- Physical page allocator.
- Memory-zone manager.
- NUMA topology manager.
- Memory controller interface.
- ECC reporting.
- Memory error handler.
- Memory scrubbing policy.
- Memory hotplug manager.
- Reserved-memory manager.
- Persistent-memory interface, if supported.
- Memory encryption interface, if supported.

### Algorithms

- Buddy allocation.
- Per-CPU page caches.
- NUMA-aware allocation.
- Page reclamation.
- Memory compaction.
- Free-page accounting.
- Fragmentation measurement.
- Memory poisoning.
- Zero-on-allocation.
- Secure reclamation.

Alternative allocation algorithms MAY be adopted when benchmarks and correctness proofs justify them.

### Required hardware parameters

- Physical address width.
- Supported page sizes.
- Cache-line size.
- Memory channel configuration.
- NUMA topology.
- ECC capabilities.
- Encryption capabilities.
- Memory bandwidth.
- Maximum supported physical memory.
- Memory-controller error semantics.

---

## 1.4 GPU platform

The GPU system has two distinct contracts:

1. Vulkan's API and execution semantics.
2. The actual hardware architecture and driver implementation.

### GPU components

- GPU discovery.
- GPU identification.
- GPU firmware loader.
- GPU reset manager.
- GPU power manager.
- GPU memory manager.
- GPU virtual-memory manager.
- GPU address-space manager.
- GPU command submission.
- GPU queue manager.
- GPU scheduler interface.
- GPU interrupt handler.
- GPU fault handler.
- GPU device-loss handler.
- GPU synchronization manager.
- GPU cache-coherency manager.
- GPU performance telemetry.
- GPU resource accounting.
- GPU isolation manager.

### Hardware details to define

- Compute units.
- Execution units.
- SIMD width.
- Warps or wavefronts.
- Register files.
- Shared memory.
- Local memory.
- Global memory.
- GPU caches.
- GPU TLBs.
- GPU page tables.
- Memory heaps.
- DMA engines.
- Command processors.
- Queue topology.
- Preemption.
- Fault recovery.
- Firmware trust.
- GPU virtualization.

### Vulkan components

CIV MUST implement or integrate:

1. Vulkan loader.
2. Instance management.
3. Physical-device enumeration.
4. Device creation.
5. Queue-family discovery.
6. Queue management.
7. Command-buffer management.
8. Command-pool management.
9. Descriptor management.
10. Descriptor-set layouts.
11. Pipeline layouts.
12. Shader-module handling.
13. Graphics-pipeline creation.
14. Compute-pipeline creation.
15. Render-target management.
16. Image management.
17. Buffer management.
18. Memory allocation.
19. Memory binding.
20. Image-layout transitions.
21. Pipeline barriers.
22. Semaphores.
23. Fences.
24. Events.
25. Swapchain management.
26. Surface management.
27. Presentation.
28. Synchronization validation.
29. Device-loss handling.
30. Extension management.
31. Validation-layer integration.
32. Shader compilation and caching.
33. Pipeline caching.

### Shader infrastructure

Define:

- SPIR-V ingestion.
- SPIR-V validation.
- Shader reflection.
- Shader compiler integration.
- Pipeline specialization.
- Shader cache.
- GPU code generation.
- Shader resource limits.
- Shader execution validation.

Vulkan does not automatically provide formal verification of shader behavior. CIV MUST specify what is verified about shader inputs, resource access, synchronization, and generated GPU instructions.

---

## 1.5 Display and input hardware

### Display components

- Display enumeration.
- EDID parser.
- Display mode manager.
- Resolution management.
- Refresh-rate management.
- Color-space management.
- HDR configuration.
- Display controller interface.
- Framebuffer management.
- Scanout buffer management.
- VSync management.
- Multi-monitor management.
- Display hotplug.
- Display power management.

### Input components

- Keyboard driver.
- Mouse driver.
- Touchscreen driver.
- Touchpad driver.
- Game-controller driver.
- HID parser.
- Input event subsystem.
- Input routing.
- Input focus manager.
- Keymap subsystem.
- Input-device security policy.

Input devices MUST be treated as untrusted sources of events.

---

## 1.6 Storage hardware

### Components

- PCIe storage discovery.
- NVMe controller initialization.
- NVMe queue management.
- NVMe submission queues.
- NVMe completion queues.
- NVMe interrupt handling.
- NVMe DMA mapping.
- Storage reset.
- Storage error reporting.
- SATA controller support, if enabled.
- Block-device abstraction.
- Storage telemetry.
- Storage power management.

### Storage algorithms

- Request scheduling.
- Queue merging.
- I/O batching.
- DMA transfer.
- Request completion.
- Timeout handling.
- Error recovery.
- Flush ordering.
- Persistent-write ordering.

---

## 1.7 Networking hardware

### Components

- Ethernet NIC discovery.
- NIC initialization.
- RX queue management.
- TX queue management.
- DMA buffer management.
- Descriptor-ring management.
- Interrupt moderation.
- Receive-side scaling.
- Checksum offload.
- Segmentation offload.
- Network device reset.
- Network link management.
- Wi-Fi device management.
- Wireless firmware management.
- Radio power management.

Each offload MUST have specified semantics and a correctness-preserving fallback.

---

## 1.8 Hardware contract registry

Every supported device MUST have an entry containing:

- Vendor.
- Device ID.
- Revision.
- Firmware version.
- Hardware capabilities.
- Required features.
- Optional features.
- Known errata.
- DMA behavior.
- Interrupt behavior.
- Reset behavior.
- Memory semantics.
- Security assumptions.
- Performance characteristics.
- Supported driver version.
- Verification status.

---

# 2. FIRMWARE AND BOOT INFRASTRUCTURE

## 2.1 Firmware interfaces

Components:

1. UEFI interface.
2. ACPI parser.
3. SMBIOS parser.
4. Firmware configuration parser.
5. Secure Boot verification.
6. Measured Boot integration.
7. TPM interface.
8. Firmware variable manager.
9. Firmware update interface.
10. Firmware recovery interface.
11. Microcode loader.
12. Firmware runtime-service isolation.

Every firmware-provided structure MUST be validated before use.

## 2.2 Bootloader

Components:

- Boot executable.
- UEFI entry point.
- Firmware memory-map reader.
- ELF64 loader.
- ELF header validator.
- ELF section validator.
- ELF segment loader.
- Relocation processor.
- Kernel image verifier.
- Kernel configuration reader.
- CPU feature discovery.
- Initial page-table builder.
- Initial stack builder.
- Boot information structure.
- Entropy initialization.
- Memory-map handoff.
- Boot error handler.
- Recovery selection.

## 2.3 Kernel initialization

Components:

- Assembly entry point.
- CPU mode validation.
- Descriptor-table initialization.
- Stack initialization.
- Kernel page-table initialization.
- Memory allocator initialization.
- Per-CPU initialization.
- Interrupt initialization.
- Timer initialization.
- Scheduler initialization.
- IOMMU initialization.
- Device discovery.
- Driver initialization.
- Security initialization.
- Cryptographic initialization.
- User-space bootstrap.

Initialization dependencies MUST be represented as an explicit dependency graph.

Circular initialization dependencies MUST be eliminated or formally resolved.

---

# 3. KERNEL CORE

CIV uses a monolithic kernel with explicit internal subsystem boundaries.

## 3.1 Kernel execution infrastructure

Components:

- Kernel entry and exit.
- CPU-local storage.
- Kernel stack management.
- Kernel execution context.
- Preemption control.
- Kernel mode transitions.
- Kernel exception handling.
- Kernel panic handling.
- Kernel assertions.
- Kernel logging.
- Kernel tracing.
- Kernel resource accounting.

## 3.2 Process manager

Components:

- Process object.
- Process identifier allocator.
- Process creation.
- Process destruction.
- Process termination.
- Process suspension.
- Process resumption.
- Process credentials.
- Process resource accounting.
- Process address-space ownership.
- Process capability table.
- Process inheritance.
- Process debugging.
- Process exit status.

## 3.3 Thread manager

Components:

- Thread object.
- Thread identifier allocator.
- Thread stack.
- Register context.
- Thread-local storage.
- Thread creation.
- Thread destruction.
- Thread blocking.
- Thread waking.
- Thread cancellation.
- Thread migration.
- Thread accounting.

## 3.4 Scheduler

Components:

- Runnable queues.
- Per-CPU run queues.
- Priority management.
- Scheduling classes.
- Time accounting.
- CPU affinity.
- CPU migration.
- Load balancing.
- Idle threads.
- Deadline scheduling.
- Real-time scheduling.
- Fair scheduling.
- Preemption.
- Priority inheritance.
- Scheduler statistics.

Candidate algorithms:

- Multilevel feedback scheduling.
- Per-CPU priority queues.
- Deadline scheduling.
- Work stealing.
- NUMA-aware load balancing.

The final algorithm MUST be selected through measured performance and formal analysis, not fashion.

## 3.5 Synchronization infrastructure

Components:

- Spinlocks.
- Mutexes.
- Read-write locks.
- Semaphores.
- Condition variables.
- Atomic primitives.
- Wait queues.
- Futex-like interfaces.
- Barriers.
- Per-CPU synchronization.
- Lock debugging.
- Lock dependency tracking.
- RCU infrastructure if adopted.
- Epoch reclamation if adopted.

Every primitive MUST define its linearization point and memory-ordering semantics.

## 3.6 Interrupt subsystem

Components:

- IDT management.
- APIC management.
- I/O APIC management.
- MSI handling.
- MSI-X handling.
- Interrupt vector allocation.
- Interrupt routing.
- Interrupt masking.
- Interrupt acknowledgment.
- Interrupt nesting.
- Interrupt affinity.
- Deferred interrupt work.
- Interrupt statistics.
- Interrupt storm detection.

## 3.7 Exception subsystem

Components:

- Page-fault handler.
- General-protection handler.
- Invalid-opcode handler.
- Double-fault handler.
- Stack-fault handling.
- Floating-point exception handling.
- Machine-check handling.
- Debug exception handling.
- NMI handling.
- User exception delivery.
- Kernel fatal-fault handling.

---

# 4. MEMORY MANAGEMENT

## 4.1 Physical memory manager

Components:

- Memory-region database.
- Physical page allocator.
- Free-page database.
- Reserved-page database.
- NUMA allocator.
- Per-CPU allocation caches.
- Page reference management.
- Page ownership management.
- Page zeroing.
- Page poisoning.
- Page reclamation.
- Memory compaction.
- Memory statistics.

## 4.2 Virtual memory manager

Components:

- Address-space object.
- Page-table allocator.
- Page-table builder.
- Page mapper.
- Page unmapper.
- Permission manager.
- TLB invalidation.
- TLB shootdown coordinator.
- Page-fault dispatcher.
- Demand paging.
- Copy-on-write.
- Shared mappings.
- Guard mappings.
- Huge-page management.
- Kernel virtual-address allocator.
- User virtual-address allocator.

## 4.3 Kernel heap

Components:

- Slab allocator.
- Size-class allocator.
- Large-object allocator.
- Per-CPU caches.
- Object constructors.
- Object destructors.
- Allocation metadata.
- Lifetime validation.
- Memory statistics.

## 4.4 User memory interfaces

Components:

- User allocation ABI.
- Mapping interfaces.
- Shared-memory interfaces.
- Memory locking.
- Memory protection.
- Memory advice.
- Memory quotas.
- Memory pressure notification.

## 4.5 Memory correctness

Formal obligations:

- No invalid page-table entry.
- No unauthorized memory mapping.
- No use-after-free.
- No double-free.
- No invalid object lifetime.
- No uninitialized sensitive-memory disclosure.
- No stale mapping after required invalidation.
- No unauthorized DMA access.
- No invalid cross-domain aliasing.

---

# 5. DEVICE AND DRIVER INFRASTRUCTURE

## 5.1 Driver framework

Components:

- Device registry.
- Device identity.
- Driver registry.
- Driver matching.
- Driver initialization.
- Driver shutdown.
- Driver reset.
- Driver dependency manager.
- Driver resource allocator.
- Driver interrupt registration.
- Driver DMA interface.
- Driver power management.
- Driver error reporting.
- Driver diagnostics.
- Driver update mechanism.

## 5.2 Device classes

The initial driver inventory MUST include:

- PCI.
- PCIe.
- NVMe.
- SATA, if supported.
- USB host controllers.
- USB hubs.
- USB HID.
- Ethernet.
- Wi-Fi.
- Bluetooth.
- Audio.
- Display.
- GPU.
- Keyboard.
- Mouse.
- Touchscreen.
- Touchpad.
- Camera.
- Sensors.
- Serial devices.
- Power controllers.
- Thermal controllers.
- TPM.

Each device class requires its own register model, command protocol, DMA model, interrupt behavior, error handling, and security contract.

## 5.3 Driver implementation rules

- Drivers MUST NOT receive unrestricted physical-memory authority.
- DMA MUST use explicit mappings.
- Hardware register access MUST use typed interfaces.
- Interrupt handlers MUST have bounded work.
- Device input MUST be treated as untrusted.
- Device reset MUST invalidate stale ownership.
- Device failure MUST not silently corrupt unrelated subsystems.

The driver registry MUST enumerate every supported device and its verification status.

---

# 6. STORAGE AND FILESYSTEM INFRASTRUCTURE

## 6.1 Block layer

Components:

- Block-device abstraction.
- Request objects.
- Submission queues.
- Completion queues.
- I/O scheduler.
- Request merger.
- DMA transfer manager.
- Flush manager.
- Error recovery.
- Device timeout manager.

## 6.2 Filesystem framework

Components:

- VFS.
- Filesystem registry.
- Mount manager.
- Namespace manager.
- Path resolver.
- Directory cache.
- Inode or object cache.
- File descriptor table.
- File handle manager.
- Permission manager.
- File locking.
- Metadata cache.
- Writeback manager.
- Filesystem error manager.

## 6.3 Initial filesystem implementation

The first native filesystem MUST define:

- On-disk format.
- Superblock.
- Allocation bitmap or equivalent.
- Inode format.
- Directory format.
- File extents.
- Checksums.
- Journaling or copy-on-write.
- Transaction semantics.
- Crash recovery.
- Metadata ordering.
- Free-space management.
- Quotas.
- Encryption integration.
- Snapshot semantics, if supported.

A second filesystem SHOULD be supported for compatibility.

## 6.4 Storage security

Define:

- Encryption at rest.
- Key lifecycle.
- Secure deletion limitations.
- File permissions.
- Directory permissions.
- Symlink behavior.
- Path traversal prevention.
- Mount isolation.
- Malformed filesystem handling.
- Crash consistency.

---

# 7. NETWORKING

## 7.1 Network device infrastructure

Components:

- NIC registry.
- RX rings.
- TX rings.
- DMA buffers.
- Packet buffers.
- Queue management.
- Interrupt moderation.
- RSS.
- Offload manager.
- Network device reset.

## 7.2 Protocol stack

Components:

- Ethernet.
- ARP.
- IPv4.
- IPv6.
- ICMP.
- ICMPv6.
- UDP.
- TCP.
- DHCP.
- DNS.
- Neighbor Discovery.
- Routing.
- Multicast.
- Socket API.

Additional protocols MAY be supported through separately specified modules.

## 7.3 Security components

- Firewall.
- Packet filter.
- Connection tracking.
- Network namespace.
- Socket permission manager.
- Rate limiter.
- Anti-spoofing validation.
- TLS library integration.
- Certificate validation.
- Network audit subsystem.

## 7.4 Network performance

Measure:

- Packet processing latency.
- Packets per second.
- Throughput.
- Memory copies.
- Allocations per packet.
- Cache misses.
- Queue contention.
- Interrupt overhead.
- Tail latency.

---

# 8. GRAPHICS, COMPUTE, AND AUDIO

## 8.1 Graphics infrastructure

Components:

- Display server.
- Window manager.
- Surface manager.
- Compositor.
- Frame scheduler.
- Input integration.
- Display configuration.
- Color management.
- GPU resource manager.
- Presentation engine.
- Vulkan loader.
- Vulkan device interface.

## 8.2 Compute infrastructure

Components:

- Vulkan compute runtime.
- SPIR-V validator.
- Shader compiler integration.
- Pipeline cache.
- Descriptor allocator.
- Compute queue manager.
- GPU synchronization manager.
- GPU memory allocator.
- GPU profiling.
- Device-loss recovery.

## 8.3 Audio infrastructure

Components:

- Audio device discovery.
- Audio driver interface.
- Audio buffer manager.
- Audio mixer.
- Audio routing.
- Audio stream API.
- Sample-rate conversion.
- Clock synchronization.
- Audio scheduling.
- Volume control.
- Device hotplug.
- Audio security policy.

Audio processing MUST define its numerical formats, timing, buffering, and real-time behavior.

---

# 9. SECURITY INFRASTRUCTURE

## 9.1 Authority system

Components:

- Principal registry.
- Security-domain registry.
- Capability representation.
- Capability allocator.
- Capability validation.
- Capability derivation.
- Capability transfer.
- Capability attenuation.
- Capability revocation.
- Object ownership.
- Authority audit.

## 9.2 Access-control engine

Components:

- Security policy database.
- Policy evaluator.
- Permission cache.
- Permission invalidation.
- Mandatory access restrictions.
- Default-deny rules.
- Domain transition manager.
- Security labels.
- Audit records.

## 9.3 Isolation infrastructure

Components:

- Address-space isolation.
- Process isolation.
- Kernel/user isolation.
- Driver authority restrictions.
- DMA isolation.
- GPU isolation.
- Filesystem isolation.
- Network isolation.
- Debugging restrictions.
- Shared-memory isolation.

## 9.4 Cryptographic infrastructure

Components:

- Cryptographic random generator.
- Entropy collector.
- Entropy estimator.
- Hash functions.
- Message authentication.
- Digital signatures.
- Key derivation.
- Symmetric encryption.
- Asymmetric cryptography.
- Key storage.
- Key rotation.
- Key destruction.
- Certificate validation.
- Secure boot verification.
- Disk encryption.
- Secure communication.

CIV MUST use reviewed cryptographic standards and independently reviewed implementations.

## 9.5 Secure execution

Components:

- W^X enforcement.
- Kernel code integrity.
- Module validation.
- Executable validation.
- ASLR.
- Stack protection.
- Control-flow protections where supported.
- Speculation mitigations.
- Debug-interface restrictions.
- Privileged-operation auditing.

## 9.6 Threat-model registry

Every security property MUST identify:

- Attacker.
- Capability.
- Target.
- Attack surface.
- Required protection.
- Assumptions.
- Verification technique.
- Residual risk.

---

# 10. MICROARCHITECTURAL SECURITY

CIV MUST explicitly model:

- Cache attacks.
- TLB attacks.
- Branch predictor attacks.
- Speculative execution.
- Speculative store bypass.
- Transient execution.
- SMT contention.
- Shared execution units.
- Performance-counter leakage.
- Memory-controller contention.
- GPU contention.
- Timer leakage.
- Page-fault leakage.
- Scheduling leakage.

Components to implement or integrate:

- CPU mitigation registry.
- Speculation policy manager.
- Cache partitioning interface where supported.
- CPU isolation policy.
- SMT policy.
- Timer-resolution policy.
- Performance-counter access control.
- Sensitive-operation scheduling.
- Constant-time cryptographic primitives.
- Side-channel test framework.

Every mitigation MUST specify which attack class it addresses.

A mitigation MUST NOT be advertised as universal protection.

---

# 11. ELECTROMAGNETIC AND PHYSICAL SECURITY

This is a separate engineering discipline.

Software cannot independently guarantee the absence of exploitable electromagnetic emissions.

CIV MUST specify both the software behavior and the physical hardware properties.

## 11.1 EM threat model

Potential sources:

- CPU switching activity.
- Memory controllers.
- DRAM buses.
- GPU execution.
- GPU memory.
- Display signaling.
- Power delivery.
- Voltage regulators.
- Clock generators.
- Peripheral controllers.
- Network interfaces.
- Internal cables.
- External cables.

Potential observations:

- Near-field emissions.
- Far-field emissions.
- Frequency-domain traces.
- Time-domain traces.
- Correlated switching activity.
- Secret-dependent power consumption.
- Device-dependent signal variation.

## 11.2 EM security components

1. EM measurement infrastructure.
2. Near-field probes.
3. Far-field measurement equipment.
4. Spectrum analysis.
5. Time-domain capture.
6. Power measurement.
7. Signal-processing pipeline.
8. Trace alignment.
9. Statistical leakage analysis.
10. Leakage regression testing.
11. Hardware configuration registry.
12. Shielding assessment.
13. Grounding assessment.
14. Cable-emission assessment.
15. Physical-security documentation.

## 11.3 EM software requirements

- Cryptographic routines MUST have defined leakage requirements.
- Sensitive operations MUST have explicit timing behavior.
- Secret-dependent control flow MUST be reviewed.
- Secret-dependent memory access MUST be reviewed.
- Power-management interactions MUST be evaluated.
- GPU execution MUST be included in relevant leakage assessments.
- Device activity MUST be included where it can expose protected information.

## 11.4 EM physical requirements

Define:

- Measurement bandwidth.
- Measurement distance.
- Antenna or probe characteristics.
- Environmental noise.
- Test configuration.
- Frequency ranges.
- Signal-to-noise ratio.
- Leakage metric.
- Acceptance threshold.
- Repeatability.
- Calibration.

The acceptance threshold MUST be derived from the actual security requirement and attacker capability.

CIV MUST NOT claim that software-level proofs establish electromagnetic confidentiality.

## 11.5 Other physical attacks

Include:

- Cold-boot attacks.
- Memory probing.
- Voltage fault injection.
- Clock fault injection.
- Debug-port attacks.
- Physical device substitution.
- Firmware extraction.
- Power analysis.
- Acoustic analysis.
- Thermal observation.
- Bus monitoring.

Every physical protection claim MUST identify whether it is software-enforced, hardware-enforced, physically measured, or assumed.

---

# 12. USER-SPACE SYSTEM

A complete OS needs more than a kernel.

## 12.1 Runtime infrastructure

Components:

- Process startup runtime.
- Thread runtime.
- Memory allocator.
- Thread-local storage runtime.
- Dynamic linker.
- Executable loader.
- System-call wrapper library.
- Synchronization library.
- Time library.
- Error-handling library.
- Cryptographic library.
- Standard library.

## 12.2 Core system services

Components:

- Service manager.
- Device enumeration service.
- Device configuration service.
- Storage service.
- Network configuration service.
- Network name-resolution service.
- Time synchronization service.
- Logging service.
- Audit service.
- Update service.
- Recovery service.
- Session manager.
- User authentication service.
- Power management service.
- Display manager.
- Audio service.

Although the kernel is monolithic, these user-space services SHOULD be separated into independent processes where doing so does not compromise the performance contract.

## 12.3 User interfaces

Components:

- Command interpreter.
- Terminal.
- Terminal emulator.
- Shell utilities.
- File manager.
- Process monitor.
- System monitor.
- Configuration manager.
- Service manager interface.
- Display settings.
- Network settings.
- Security settings.
- Accessibility interfaces.

## 12.4 Application interfaces

Define:

- System-call ABI.
- Runtime ABI.
- Dynamic-linking ABI.
- Executable format.
- Threading API.
- Filesystem API.
- Network API.
- Graphics API.
- Compute API.
- Audio API.
- Input API.
- Security API.
- Time API.
- IPC API.

All APIs require versioning and compatibility rules.

---

# 13. COMPILER AND DEVELOPMENT TOOLCHAIN

CIV's compiler toolchain is part of its trusted computing boundary.

## 13.1 Compiler infrastructure

Components:

- Rust compiler toolchain.
- Rust standard-library subset.
- Target configuration.
- Linker.
- Assembler.
- Binary utilities.
- ELF parser.
- ELF linker.
- Symbol processor.
- Relocation processor.
- Disassembler.
- Debug-information processor.
- Static analyzer.
- Linter.
- Dependency manager.

## 13.2 Assembly infrastructure

Define:

- Calling conventions.
- Register usage.
- Stack alignment.
- Interrupt conventions.
- Context-switch conventions.
- Inline assembly contracts.
- Memory clobbers.
- Control-flow constraints.
- Instruction legality.
- CPU feature requirements.

## 13.3 Binary verification

Components:

- ELF verifier.
- Section permission verifier.
- Relocation verifier.
- Symbol verifier.
- ABI verifier.
- W^X verifier.
- Binary integrity verifier.
- Machine-code inspection.
- Disassembly comparison.
- Final image hash verification.

## 13.4 Reproducible builds

Components:

- Hermetic build environment.
- Dependency lockfiles.
- Compiler version registry.
- Linker version registry.
- Prover version registry.
- Build configuration.
- Generated-source registry.
- Artifact hashing.
- Build provenance.
- Signing infrastructure.
- Reproducibility checker.
- Software bill of materials.

Identical declared source, toolchain, and configuration inputs SHOULD produce identical output artifacts.

---

# 14. FORMAL VERIFICATION INFRASTRUCTURE

## 14.1 Mathematical specification

Components:

- Machine-state definitions.
- Transition relations.
- Memory model.
- Ownership model.
- Authority model.
- Security labels.
- Resource model.
- Timing model.
- Failure model.
- Hardware assumptions.

## 14.2 Prover infrastructure

Primary tools:

- Lean 4.
- Rocq/Coq.
- Isabelle/HOL.
- F*.
- SMT solvers.
- SAT solvers.
- TQBF solvers.
- TLA+ model checking.
- Alloy model exploration.
- CBMC.
- KLEE.
- Differential testing.

Each tool MUST have an explicit verification responsibility.

## 14.3 Proof registry

Every proof artifact MUST record:

- Proof ID.
- Requirement ID.
- Specification version.
- Implementation revision.
- Tool version.
- Tool configuration.
- Assumptions.
- Dependencies.
- Proof result.
- Proof scope.
- Artifact hash.
- Reviewer.
- Reproduction instructions.

## 14.4 Refinement verification

The project MUST establish refinement across:

1. Mathematical specification.
2. Abstract kernel.
3. Concrete implementation.
4. Compiler output.
5. Linked binary.
6. Booted system.

The deployed system MUST be shown to satisfy the specified properties under the declared hardware and environmental assumptions.

A proof of the abstract kernel alone is insufficient.

---

# 15. TQBF AND ADVERSARIAL ANALYSIS

## 15.1 Formal exploit model

Define:

- Initial state.
- Attacker capabilities.
- System transition relation.
- Environmental behavior.
- Vulnerability predicate.
- Execution bound.
- Security objective.

A bounded exploit query takes the form:

EXISTS initial state, attacker strategy, execution trace:

ValidInitial(state)

AND

Reachable(state, attacker, trace)

AND

Vulnerable(trace)

A satisfying assignment represents an exploit within the modeled system and bound.

An unsatisfiable result excludes the modeled exploit within that bound and model.

It does not establish universal real-world exploit resistance.

## 15.2 Quantified attacker strategies

TQBF MUST be used where the attacker and system alternate decisions.

The quantifier order MUST model the actual information available to each participant.

Example:

FORALL attacker action:

EXISTS permitted defensive response:

INVARIANT remains true.

More complicated properties MUST explicitly model:

- Attacker knowledge.
- Execution order.
- Hidden state.
- Scheduling.
- Resource limits.
- Device responses.
- System reactions.

## 15.3 SMT verification

Required theories:

- Bit vectors.
- Integer arithmetic.
- Arrays.
- Memory models.
- Address ranges.
- Alignment.
- Pointer constraints.
- Resource constraints.
- Permission constraints.
- Timing constraints where applicable.

## 15.4 Adversarial scenarios

The engine MUST investigate:

- Privilege escalation.
- Invalid system calls.
- Memory corruption.
- Capability forgery.
- Capability escalation.
- DMA attacks.
- IOMMU misconfiguration.
- GPU memory violations.
- Interrupt races.
- Scheduler races.
- Locking failures.
- Integer overflow.
- Parser vulnerabilities.
- Filesystem corruption.
- Malformed firmware.
- Malformed device responses.
- Resource exhaustion.
- Power interruption.
- Device reset races.
- Side-channel leakage.
- EM leakage.

## 15.5 Counterexample infrastructure

Components:

- Trace recorder.
- State serializer.
- Counterexample minimizer.
- Reproduction generator.
- Regression-test generator.
- Vulnerability classifier.
- Proof-failure reporter.

Every confirmed exploit MUST become a permanent regression artifact.

---

# 16. PERFORMANCE ENGINEERING

Performance is a system requirement, not an afterthought.

## 16.1 Performance instrumentation

Components:

- CPU cycle measurement.
- Hardware performance-counter interface.
- Cache statistics.
- TLB statistics.
- Branch statistics.
- Context-switch statistics.
- Scheduler statistics.
- Allocation statistics.
- NUMA statistics.
- Interrupt statistics.
- GPU profiling.
- Storage profiling.
- Network profiling.
- Power measurement.
- Thermal measurement.

## 16.2 Performance contracts

Every subsystem MUST define:

- Latency.
- Throughput.
- Memory overhead.
- CPU overhead.
- Allocation count.
- Copy count.
- Synchronization overhead.
- Cache behavior.
- Tail latency.
- Worst-case behavior under stated assumptions.

## 16.3 Optimization requirements

The following MUST be investigated:

- Per-CPU data structures.
- Cache-line alignment.
- False-sharing elimination.
- Batching.
- Zero-copy data transfer.
- Reduced privilege transitions.
- Reduced allocation.
- Reduced memory copying.
- NUMA locality.
- Efficient TLB management.
- Lock contention reduction.
- Fast-path specialization.
- Branch prediction behavior.
- SIMD utilization.
- GPU submission batching.
- Asynchronous I/O.
- Efficient interrupt moderation.

## 16.4 Optimization verification

Every performance optimization MUST have:

1. A correctness specification.
2. A proof or refinement argument.
3. A baseline benchmark.
4. An optimized benchmark.
5. A workload definition.
6. A hardware definition.
7. A statistical report.
8. A regression threshold.

No optimization may silently weaken a security invariant.

---

# 17. RELIABILITY AND FAULT MANAGEMENT

## 17.1 Failure classes

Define:

- Application crash.
- Thread crash.
- Driver failure.
- Kernel assertion failure.
- Kernel panic.
- CPU machine check.
- ECC memory error.
- Storage failure.
- GPU timeout.
- GPU device loss.
- Network failure.
- Power failure.
- Firmware failure.
- Invalid device response.
- Resource exhaustion.

## 17.2 Recovery components

- Kernel panic handler.
- Crash recorder.
- Crash dump writer.
- Watchdog.
- Driver reset manager.
- GPU recovery manager.
- Storage recovery manager.
- Boot recovery manager.
- Update rollback.
- Filesystem recovery.
- Diagnostic collector.

## 17.3 Recovery guarantees

Every recoverable failure MUST define:

- Detection.
- Containment.
- State preservation.
- Cleanup.
- Recovery procedure.
- Recovery timeout.
- Recovery failure behavior.

A failure MUST NOT be declared recoverable unless the corresponding recovery path is implemented and tested.

---

# 18. SECURITY UPDATES AND MAINTENANCE

Components:

- Update client.
- Update manifest.
- Signature verification.
- Package validator.
- Dependency resolver.
- Kernel update mechanism.
- Driver update mechanism.
- Firmware update mechanism.
- Rollback prevention.
- Recovery partition.
- Key rotation.
- Key revocation.
- Vulnerability database.
- Security advisory mechanism.
- Update audit log.

Updates MUST preserve compatibility and establish the verification status of the newly deployed artifact.

---

# 19. TESTING INFRASTRUCTURE

## 19.1 Unit testing

Every subsystem MUST have unit tests.

## 19.2 Integration testing

Required:

- Boot-to-kernel.
- Kernel-to-driver.
- Kernel-to-memory.
- Kernel-to-GPU.
- Kernel-to-storage.
- Kernel-to-network.
- Kernel-to-filesystem.
- User-space-to-kernel.
- Update-to-recovery.

## 19.3 Hardware testing

Required:

- CPU feature tests.
- Paging tests.
- TLB tests.
- Cache-coherence tests.
- Interrupt tests.
- DMA tests.
- IOMMU tests.
- GPU isolation tests.
- Storage fault tests.
- Network fault tests.
- Thermal tests.
- Power interruption tests.

## 19.4 Adversarial testing

Required:

- Coverage-guided fuzzing.
- Mutation fuzzing.
- Differential testing.
- Fault injection.
- Race testing.
- Stress testing.
- Malformed hardware inputs.
- Malformed firmware inputs.
- Protocol abuse.
- Side-channel testing.

## 19.5 Physical testing

Required:

- Power analysis.
- EM analysis.
- Thermal analysis.
- Acoustic analysis where relevant.
- Fault-injection evaluation.
- Shielding assessment.
- Physical tampering assessment.

---

# 20. DOCUMENTATION AND ENGINEERING GOVERNANCE

Components:

- Master specification.
- Architecture decision records.
- Hardware compatibility registry.
- Interface registry.
- Requirement registry.
- Proof registry.
- Assumption registry.
- Threat registry.
- Performance registry.
- Dependency registry.
- Driver registry.
- Test registry.
- Vulnerability registry.
- Residual-risk registry.
- Release registry.

Every component MUST have an owner.

Every architectural change MUST identify affected requirements and proofs.

Every release MUST identify the exact source revision, build configuration, toolchain, hardware profile, and verification evidence.

---

# 21. REQUIRED SYSTEM-WIDE PROOFS

The following are mandatory top-level verification objectives.

### Memory

- Memory safety.
- Ownership correctness.
- Mapping correctness.
- Permission correctness.
- Lifetime correctness.
- DMA isolation.

### Execution

- Privilege separation.
- Context-switch correctness.
- Exception correctness.
- Interrupt correctness.
- Scheduler safety.
- Concurrency safety.

### Security

- Capability integrity.
- Access-control correctness.
- Information-flow security.
- Cryptographic implementation correctness.
- Device isolation.
- GPU isolation.
- Speculative-execution security within the modeled scope.

### Reliability

- Failure containment.
- Recovery correctness.
- Crash consistency.
- Update correctness.
- Resource accounting.

### Physical security

- EM leakage characterization.
- Power leakage characterization.
- Physical attack assumptions.
- Fault-injection resilience.
- Hardware trust assumptions.

### Implementation

- Source-to-binary correctness.
- ABI correctness.
- Linker correctness or validated output.
- Boot-chain integrity.
- Reproducible build evidence.

---

# 22. RELEASE ACCEPTANCE CRITERIA

A CIV release MUST NOT be declared fully verified unless:

1. All mandatory requirements have an implementation.
2. All mandatory formal obligations are discharged.
3. All hardware assumptions are documented.
4. All supported hardware profiles are tested.
5. All required security boundaries are validated.
6. All critical drivers have completed their verification requirements.
7. All system-wide invariants hold.
8. All required adversarial tests pass.
9. All required physical leakage assessments are complete.
10. All performance contracts are measured.
11. The release binary has been validated.
12. The build is reproducible under the declared build environment.
13. No unresolved critical vulnerability remains.
14. All residual risks are documented.
15. The exact release configuration is recorded.

An incomplete proof MUST NOT be represented as a completed proof.

An untested hardware profile MUST NOT be represented as supported.

A measured absence of leakage in one experiment MUST NOT be represented as a universal mathematical guarantee.

---

# 23. MASTER COMPONENT COMPLETENESS RULE

CIV is considered architecturally specified only when every component has:

- A unique component identifier.
- A defined purpose.
- A concrete implementation plan.
- An interface specification.
- A data model.
- An algorithm selection.
- A hardware dependency declaration.
- A memory ownership model.
- A security contract.
- A concurrency contract.
- A performance contract.
- A failure model.
- A test plan.
- A formal verification plan.
- An implementation status.
- An acceptance criterion.

A component without these fields is not complete.

The specification MUST expand whenever a new hardware dependency, subsystem, protocol, interface, or attack surface is introduced.

---

# END OF ARCHITECTURAL BASELINE

CIV-MASTER-001 Revision 1.1

Status: Architecture established. Detailed component specifications, hardware profiles, implementations, and proofs remain to be produced.
