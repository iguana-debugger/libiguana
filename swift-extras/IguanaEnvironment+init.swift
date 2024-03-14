//
//  IguanaEnvironment+init.swift
//  
//
//  Created by James on 22/02/2024.
//

import Foundation

public extension IguanaEnvironment {
    /// Initialises `IguanaEnvironment` using the bundled `jimulator` and `aasm` executable.
    public convenience init() throws {
        let jimulatorPath = Bundle.module.path(forResource: "jimulator", ofType: nil)!
        let aasmPath = Bundle.module.path(forResource: "aasm", ofType: nil)!
        let mnemonicsPath = Bundle.module.path(forResource: "mnemonics", ofType: nil)!
        
        try self.init(jimulatorPath: jimulatorPath, aasmPath: aasmPath, mnemonicsPath: mnemonicsPath)
    }
}
