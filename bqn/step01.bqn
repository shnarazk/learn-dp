#!/usr/bin/env cbqn
lib ← •Import "lib.bqn"
NewVariable ⇐ {F init: 
  val ⇐ init
  Data ⇐ { 𝕤 ⋄ val }
  Set ⇐ { F v: val ↩ v }
}

x ← NewVariable 1
"direct access to data"   lib.Debug x.val
"access it by a mehod"    lib.Debug x.Data @
# update the stored value 
x.Set 2
"display updated data"    lib.Debug x.Data @
