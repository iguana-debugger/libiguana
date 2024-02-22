//
//  IguanaEnvironment+init.swift
//  
//
//  Created by James on 22/02/2024.
//

import Foundation

public extension IguanaEnvironment {
    /// Initialises `IguanaEnvironment` using the bundled `jimulator` executable.
    public convenience init() throws {
        let path = Bundle.module.path(forResource: "jimulator", ofType: nil)!
        
        self.init(path: path)
    }
}
