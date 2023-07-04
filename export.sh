#!/usr/bin/env bash

# Tool is located at https://github.com/Simbotic/FBX2glTF

# Body
FBX2glTF --binary --input assets/npc/Animation_rig/Body.fbx --output assets/npc/Animation_rig/Body.glb
FBX2glTF --binary --input assets/npc/Body/Body_Blue_001.fbx --output assets/npc/Body/Body_Blue_001.glb

# Hair
FBX2glTF --binary --input assets/npc/Hair/Hair_001.fbx --output assets/npc/Hair/Hair_001.glb

# Animations
FBX2glTF --binary --input assets/npc/Animation_rig/Skinning_Test.fbx --output assets/npc/Animation_rig/Skinning_Test.glb
