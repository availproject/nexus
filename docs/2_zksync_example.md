Here's the content for the `zksync_example.md`:

### **`docs/zksync_example.md`**

```markdown
# ZKSync Example with Avail Nexus

This guide will help you set up and run the ZKSync adapter with Avail Nexus.

## Prerequisites

### 1. Clone the ZKSync Repository

- Clone the ZKSync repository:

```bash
git clone https://github.com/vibhurajeev/zksync-era.git
cd zksync-era
```

### 2. Follow the Development Setup
- Complete the development setup.
- Install the zksync cli for managing zksync services:

```bash
./bin/zkt
```

- Initialize the zkStack ecosystem:

```bash
zksync_inception ecosystem init
```

## Running the ZKSync Server

- Start the ZKSync server to run the chain:

```bash
zksync_inception server
```

## Running the ZKSync Adapter with Avail Nexus

### 1. Ensure Nexus Server is Running

- Make sure the Nexus server is running:

```zsh
nexus_cli nexus --dev
```

### 2. Ensure Zksync Server is Running

- Make sure the Zksync server is running as per the previous steps:

### 3. Run the ZKSync Adapter

- Run the ZKSync adapter with default ports and Nexus account IDs:

```zsh
nexus_cli zksync
```
