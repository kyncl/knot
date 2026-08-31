# Knots and Their Types
Throughout this documentation, directories are referred to as
**Knots**. A Knot is a logical unit representing a specific
directory and its connection properties. There are two primary
types of Knots:

* **Source:** The core of the synchronization process. You
  must define one source Knot, which is typically your local
  directory.
* **Remote:** The synchronization targets. You can define
  multiple remote Knots to synchronize against the source.

Advanced features, such as Behaviors, rely on these defined
roles.

This structure resembles a star network topology
([Figure 1](#figure-1)). However, to ensure data safety, Knot
synchronizes the source with each remote Knot sequentially
rather than concurrently.

Concurrent synchronization across multiple remotes could cause
race conditions if the same file is modified in two distinct
remote Knots. Processing remotes sequentially establishes a
clear priority and prevents unresolvable conflicts.

<div class="workflow-diagram" style="border-radius: 4px; margin: 2rem 0; overflow-x: auto; color: white;" id="figure-1">
<svg viewBox="0 0 800 400" width="100%" style="min-width: 545px; display: block;" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <marker id="arrowhead-gray-solid" markerWidth="6" markerHeight="6" refX="4" refY="3" orient="auto">
            <polygon points="0 1, 5 3, 0 5" fill="var(--muted)" />
        </marker>
    </defs>

<!-- Main Container -->
<rect x="10" y="10" width="780" height="380" rx="6" fill="#111" stroke="var(--border)" stroke-width="1"/>

<!-- Title Badge -->
<rect x="25" y="20" width="165" height="22" rx="3" fill="#18181b" stroke="var(--border)"/>
<text x="107.5" y="31" font-family="var(--font-code)" font-size="11" fill="white" text-anchor="middle" dominant-baseline="middle">Synchronization Knots</text>

<!-- Legend -->
<rect x="635" y="20" width="135" height="22" rx="3" fill="#18181b" stroke="var(--border)"/>
<text x="702.5" y="31" font-family="var(--font-code)" font-size="10" fill="var(--muted)" text-anchor="middle" dominant-baseline="middle">R1-R8 = Remote Knots</text>

<!-- Connection Lines (Drawn first to stay behind the shapes) -->
<!-- To R1 (Top) -->
<line x1="400" y1="160" x2="400" y2="82" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R2 (Top Right) -->
<line x1="426" y1="181" x2="527" y2="109" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R3 (Right) -->
<line x1="450" y1="200" x2="598" y2="200" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R4 (Bottom Right) -->
<line x1="426" y1="219" x2="527" y2="291" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R5 (Bottom) -->
<line x1="400" y1="240" x2="400" y2="318" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R6 (Bottom Left) -->
<line x1="374" y1="219" x2="273" y2="291" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R7 (Left) -->
<line x1="350" y1="200" x2="202" y2="200" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />
<!-- To R8 (Top Left) -->
<line x1="374" y1="181" x2="273" y2="109" stroke="var(--muted)" stroke-width="1.5" marker-end="url(#arrowhead-gray-solid)" />

<!-- Nodes -->

<!-- Central Hub (Source Knot) - Widened slightly to fit the text -->
<polygon points="400,160 450,200 400,240 350,200" fill="rgba(184, 42, 32, 0.15)" stroke="var(--main)" stroke-width="2"/>
<text x="400" y="201" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">Source</text>

<!-- Peripheral Nodes (R1 to R8, starting top and going clockwise) -->

<!-- R1 -->
<polygon points="400,38 422,60 400,82 378,60" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="400" y="61" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R1</text>

<!-- R2 -->
<polygon points="540,78 562,100 540,122 518,100" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="540" y="101" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R2</text>

<!-- R3 -->
<polygon points="620,178 642,200 620,222 598,200" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="620" y="201" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R3</text>

<!-- R4 -->
<polygon points="540,278 562,300 540,322 518,300" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="540" y="301" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R4</text>

<!-- R5 -->
<polygon points="400,318 422,340 400,362 378,340" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="400" y="341" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R5</text>

<!-- R6 -->
<polygon points="260,278 282,300 260,322 238,300" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="260" y="301" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R6</text>

<!-- R7 -->
<polygon points="180,178 202,200 180,222 158,200" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="180" y="201" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R7</text>

<!-- R8 -->
<polygon points="260,78 282,100 260,122 238,100" fill="rgba(217, 102, 105, 0.15)" stroke="var(--main-alt)" stroke-width="2"/>
<text x="260" y="101" font-family="var(--font-code)" font-size="12" font-weight="600" fill="white" text-anchor="middle" dominant-baseline="middle">R8</text>
</svg>
</div>

## Knot Connection Types

While a Knot's *type* (Source or Remote) defines its role in
the topology, its *connection type* defines how Knot
communicates with that directory.

### Local
Requires connection credentials: **False**

The Local connection type targets a directory on your host
machine. It requires no credentials or external resources. You
will primarily use this for your local source Knot. The only
required configuration is the directory path.

### SSH
Requires connection credentials: **True**

SSH is the most performant remote connection method. It requires
the `knot` binary to be installed on the target remote device.
Knot utilizes SSH multiplexing to distribute workloads across
multiple connections, maximizing throughput.

> [!WARNING]
> SSH connections to Windows Server environments are currently
> untested. While Knot handles OS operations natively,
> unforeseen platform-specific nuances may cause malfunctions.
> Proceed with caution.

### SFTP
Requires connection credentials: **True**

SFTP support is currently under development. 🏗️
