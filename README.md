## Overview

This is a simple POC that copies operator index images in dockerv2 format (from a registry) to disk
and then traverses these catalogs to make a best effort upgrade path for a specific operator

## Use Case

A typical use case could be:
- Work with catalog i.e registry.redhat.io/redhat/redhat-operator-index:v4.12
- Via cli find the upgrade path for a specific operator
- Add more functionality i.e as an option use a specifc channel

## POC 

I used a simple approach - Occam's razor

- A scientific and philosophical rule that entities should not be multiplied unnecessarily (KISS)
- Worked with a v2 images for the POC

[![asciicast](https://asciinema.org/a/gxdY9uyvd3WxhKVhD4PPfguUx.svg)](https://asciinema.org/a/gxdY9uyvd3WxhKVhD4PPfguUx)


## Usage

Clone this repo

Ensure that you have the correct permissions set in the $XDG_RUNTIME_DIR/containers/auth.json file

Execute the following to copy and calculate upgrade paths for several
catalogs and specific packages

```bash
mkdir -p working-dir/rhopi/blobs/sha256
cargo build 

# create a filter config (this uses the example in this repo)
kind: FilterConfiguration
apiVersion: mirror.openshift.io/v1alpha2
catalogs: 
- registry.redhat.io/redhat/redhat-operator-index:v4.12
- registry.redhat.io/redhat/redhat-operator-index:v4.13
packages:
  - name: aws-load-balancer-operator
    channel: stable-v1
    fromVersion: 0.2.0

# execute 
cargo run -- --config filter-config.yaml 
```

## Reference : OLM update graph documentation

**Replaces**

For explicit updates from one operator version to another, you can specify the operator name to replace in your channel entry as such:

```
---
schema: olm.channel
package: myoperator
channel: stable
entries:
  - name: myoperator.v1.0.1
    replaces: myoperator.v1.0.0
```

Note that it is not required for there to be an entry for myoperator.v1.0.0 in the catalog as long as other channel invariants (verified by opm validate) still hold. Generally, this means that the tail of the channel’s replaces chain can replace a bundle that is not present in the catalog.

An update sequence of bundles created via replaces will have updates step through each version in the chain. For example, given

myoperator.v1.0.0 -> myoperator.v1.0.1 -> myoperator.v1.0.2
A subscription on myoperator.v1.0.0 will update to myoperator.v1.0.2 through myoperator.v1.0.1.

Installing from the UI today will always install the latest of a given channel. However, installing specific versions is possible with this update graph by modifying the startingCSV field of the subscription to point to the desired operator name. Note that, in this case, the subscription will need its approval field set to Manual to ensure that the subscription does not auto-update and instead stays pinned to the specified version.


**Skips**

In order to skip through certain updates, you can specify a list of operator names to be skipped. For example:


```
---
schema: olm.channel
package: myoperator
channel: stable
entries:
  - name: myoperator.v1.0.3
    replaces: myoperator.v1.0.0
    skips:
      - myoperator.v1.0.1
      - myoperator.v1.0.2
```

Using the above graph, this will mean subscriptions on myoperator.v1.0.0 can update directly to myoperator.v1.0.3 without going through myoperator.v1.0.1 or myoperator.v1.0.2. Installs that are already on myoperator.v1.0.1 or myoperator.v1.0.2 will still be able to update to myoperator.v1.0.3.

This is particularly useful if myoperator.v1.0.1 and myoperator.v1.0.2 are affected by a CVE or contain bugs.

Skipped operators do not need to be present in a catalog or set of manifests prior to adding to a catalog.

**SkipRange**

OLM also allows you to specify updates through version ranges in your channel entry. This requires your CSVs to define a version in their version field which must follow the semver spec. Internally, OLM uses the blang/semver go library.

```
---
schema: olm.channel
package: myoperator
name: stable
entries:
  - name: myoperator.v1.0.3
    skipRange: ">=1.0.0 <1.0.3"
```

The entry specifying the skipRange will be presented as a direct (one hop) update to any version from that package within that range. The versions in this range do not need to be in the catalog in order for bundle addition to be successful. We recommend avoiding using unbound ranges such as <1.0.3.

WARNING: As a consequence of using skipRange the skipped operator versions will be pruned from OLMs update graph and therefore will not be installable anymore by users with the spec.startingCSV property of Subscriptions. Use with caution. If you want to have direct (one hop) updates to this version from multiple previous versions but keep those previous versions available to users for install, always use skipRange in conjunction with replaces pointing to the immediate previous version of the operator version in question.

SkipRange by itself is useful for teams who are not interested in supporting directly installing versions within a given range or for whom consumers of the operator are always on the latest version.
