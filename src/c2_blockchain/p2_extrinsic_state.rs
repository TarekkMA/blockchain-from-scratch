//! Now that we have a functioning hash-linked data structure, we can use it to actually
//! track some state. Here we will start to explore the idea of extrinsics and state by
//! slightly abusing the header's extrinsics_root and state_root fields. As the names imply,
//! these are typically used for Merkle roots of large data sets. But in our case we will use
//! these fields to directly contain a single extrinsic per block, and a single piece of state.
//!
//! In the coming parts of this tutorial, we will expand this to be more real-world like and
//! use some real batching.

use std::process::Child;

use crate::hash;

// We will use Rust's built-in hashing where the output type is u64. I'll make an alias
// so the code is slightly more readable.
type Hash = u64;

/// The header is now expanded to contain an extrinsic and a state. Note that we are not
/// using roots yet, but rather directly embedding some minimal extrinsic and state info
/// into the header.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Header {
    parent: Hash,
    height: u64,
    extrinsic: u64,
    state: u64,
    // Still no consensus. That's the next part.
    consensus_digest: (),
}

// Here are the methods for creating new header and verifying headers.
// It is your job to write them.
impl Header {
    /// Returns a new valid genesis header.
    fn genesis() -> Self {
        Header {
            parent: 0,
            height: 0,
            extrinsic: 0,
            state: 0,
            consensus_digest: (),
        }
    }

    /// Create and return a valid child header.
    fn child(&self, extrinsic: u64) -> Self {
        let parent_hash = hash(&self);
        Header {
            parent: parent_hash,
            height: self.height + 1,
            extrinsic,
            state: self.state + extrinsic,
            consensus_digest: (),
        }
    }

    /// Verify that all the given headers form a valid chain from this header to the tip.
    ///
    /// In addition to the consecutive heights and linked hashes, we now need to consider our state.
    /// This blockchain will work as an adder. That means that the state starts at zero,
    /// and at each block we add the extrinsic to the state.
    ///
    /// So in order for a block to verify, we must have that relationship between the extrinsic,
    /// the previous state, and the current state.
    fn verify_sub_chain(&self, chain: &[Header]) -> bool {
        let mut current_block = self;
        for block in chain {
            if block.height != current_block.height + 1 {
                return false;
            }
            if block.parent != hash(current_block) {
                return false;
            }
            if block.state != current_block.state + block.extrinsic {
                return false;
            }
            current_block = block;
        }
        true
    }
}

// And finally a few functions to use the code we just

/// Build and return a valid chain with the given number of blocks.
fn build_valid_chain(n: u64) -> Vec<Header> {
    let mut chain = vec![Header::genesis()];
    for i in 0..n {
        chain.push(chain[i as usize].child(i));
    }
    chain
}

/// Build and return a chain with at least three headers.
/// The chain should start with a proper genesis header,
/// but the entire chain should NOT be valid.
///
/// As we saw in the last unit, this is trivial when we construct arbitrary blocks.
/// However, from outside this crate, it is not so trivial. Our interface for creating
/// new blocks, `genesis()` and `child()`, makes it impossible to create arbitrary blocks.
///
/// For this function, ONLY USE the the `genesis()` and `child()` methods to create blocks.
/// The exercise is still possible.
fn build_an_invalid_chain() -> Vec<Header> {
    let mut chain = vec![Header::genesis()];
    chain.push(chain[0].child(1));
    chain.push(chain[0].child(2));
    chain.push(chain[2].child(4));
    chain
}

/// Build and return two header chains.
/// Both chains should individually be valid.
/// They should have the same genesis header.
/// They should not be the exact same chain.
///
/// Here is an example of two such chains:
///            /-- 3 -- 4
/// G -- 1 -- 2
///            \-- 3'-- 4'
///
/// Side question: What is the fewest number of headers you could create to achieve this goal.
fn build_forked_chain() -> (Vec<Header>, Vec<Header>) {
    let g = Header::genesis();
    let h1 = g.child(1);
    let h2 = h1.child(2);

    let mut chain1 = vec![g.clone(), h1.clone(), h2.clone()];
    let mut chain2 = vec![g, h1, h2];

    chain1.push(chain1[2].child(55));
    chain1.push(chain1[3].child(545));

    chain2.push(chain2[2].child(34));
    chain2.push(chain2[3].child(23));

    (chain1, chain2)
}

// To run these tests: `cargo test bc_2`
#[test]
fn bc_2_genesis_block_height() {
    let g = Header::genesis();
    assert!(g.height == 0);
}

#[test]
fn bc_2_genesis_block_parent() {
    let g = Header::genesis();
    assert!(g.parent == 0);
}

#[test]
fn bc_2_genesis_block_extrinsic() {
    // Typically genesis blocks do not have any extrinsics.
    // In Substrate they never do. So our convention is to have the extrinsic be 0.
    let g = Header::genesis();
    assert!(g.extrinsic == 0);
}

#[test]
fn bc_2_genesis_block_state() {
    let g = Header::genesis();
    assert!(g.state == 0);
}

#[test]
fn bc_2_child_block_height() {
    let g = Header::genesis();
    let b1 = g.child(0);
    assert!(b1.height == 1);
}

#[test]
fn bc_2_child_block_parent() {
    let g = Header::genesis();
    let b1 = g.child(0);
    assert!(b1.parent == hash(&g));
}

#[test]
fn bc_2_child_block_extrinsic() {
    let g = Header::genesis();
    let b1 = g.child(7);
    assert_eq!(b1.extrinsic, 7);
}

#[test]
fn bc_2_child_block_state() {
    let g = Header::genesis();
    let b1 = g.child(7);
    assert_eq!(b1.state, 7);
}

#[test]
fn bc_2_verify_genesis_only() {
    let g = Header::genesis();

    assert!(g.verify_sub_chain(&[]));
}

#[test]
fn bc_2_verify_three_blocks() {
    let g = Header::genesis();
    let b1 = g.child(5);
    let b2 = b1.child(6);

    assert_eq!(b2.state, 11);
    assert!(g.verify_sub_chain(&[b1, b2]));
}

#[test]
fn bc_2_cant_verify_invalid_parent() {
    let g = Header::genesis();
    let mut b1 = g.child(5);
    b1.parent = 10;

    assert!(!g.verify_sub_chain(&[b1]));
}

#[test]
fn bc_2_cant_verify_invalid_number() {
    let g = Header::genesis();
    let mut b1 = g.child(5);
    b1.height = 10;

    assert!(!g.verify_sub_chain(&[b1]));
}

#[test]
fn bc_2_cant_verify_invalid_state() {
    let g = Header::genesis();
    let mut b1 = g.child(5);
    b1.state = 10;

    assert!(!g.verify_sub_chain(&[b1]));
}

#[test]
fn bc_2_invalid_chain_is_really_invalid() {
    // This test chooses to use the student's own verify function.
    // This should be relatively safe given that we have already tested that function.
    let invalid_chain = build_an_invalid_chain();
    assert!(!invalid_chain[0].verify_sub_chain(&invalid_chain[1..]))
}

#[test]
fn bc_2_verify_forked_chain() {
    let g = Header::genesis();
    let (c1, c2) = build_forked_chain();

    // Both chains have the same valid genesis block
    assert_eq!(g, c1[0]);
    assert_eq!(g, c2[0]);

    // Both chains are individually valid
    assert!(g.verify_sub_chain(&c1[1..]));
    assert!(g.verify_sub_chain(&c2[1..]));

    // The two chains are not identical
    // Question for students: I've only compared the last blocks here.
    // Is that enough? Is it possible that the two chains have the same final block,
    // but differ somewhere else?
    assert_ne!(c1.last(), c2.last());
}
