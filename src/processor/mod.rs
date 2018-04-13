use failure::Error;

/*
Ingredient
Tool
Container
Appliance

Verb
Combine
Transfer
Measure
Take
Leave
Place
Remove
Configure
Meld
Discard
Empty
TagSet
TagGet

Simultaneous
Modifier
Annotation
Time
Until
*/

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProcessItem {
    // Concrete
    Ingredient {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },
    Tool {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },
    Container {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },
    Appliance {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },

    // Operator
    Verb {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },
    Combine {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },
    Transfer {
        name: String,
        mods: Vec<String>,
        anns: Vec<String>,
    },
    Measure {
        quantity: Quantity,
    },
    Take {
        portion: Portion,
    },
    Leave {
        portion: Portion,
    },
    Place,
    Remove,
    Configure {
        name: String,
    },
    Meld,
    Discard,
    Empty,
    TagSet {
        tag: String,
    },
    TagGet {
        tag: String,
    },

    // Meta
    Modifier {
        name: String,
    },
    Annotation {
        name: String,
    },
    // Simultaneous,
}

impl ProcessItem {
    pub fn apply_modifier(&mut self, modifier: String) -> Result<(), Error> {
        match self {
            ProcessItem::Ingredient{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            ProcessItem::Tool{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            ProcessItem::Container{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            ProcessItem::Appliance{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            ProcessItem::Verb{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            ProcessItem::Combine{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            ProcessItem::Transfer{ name: _, ref mut mods, anns: _ } => {
                mods.push(modifier);
                Ok(())
            },
            _ => {
                bail!(ProcessError::NotModifiable{target: self.clone()});
            }
        }
    }

    pub fn apply_annotation(&mut self, annotation: String) -> Result<(), Error> {
        match self {
            ProcessItem::Ingredient{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            ProcessItem::Tool{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            ProcessItem::Container{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            ProcessItem::Appliance{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            ProcessItem::Verb{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            ProcessItem::Combine{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            ProcessItem::Transfer{ name: _, ref mut anns, mods: _ } => {
                anns.push(annotation);
                Ok(())
            },
            _ => {
                bail!(ProcessError::NotAnnotatable{target: self.clone()});
            }
        }
    }

    pub fn process_meta_level<II: IntoIterator<Item = Self>>(tokens: II) -> Result<Vec<Self>, Error> {
        let mut stack: Vec<Self> = vec![];

        let tokens = tokens.into_iter();

        for token in tokens {
            match token {
                ProcessItem::Modifier { name } => {
                    if let Some(mut target) = stack.pop() {
                        target.apply_modifier(name)?;
                    }
                    else {
                        bail!(ProcessError::Empty);
                    }
                }
                ProcessItem::Annotation { name } => {
                    if let Some(mut target) = stack.pop() {
                        target.apply_annotation(name)?;
                    }
                    else {
                        bail!(ProcessError::Empty);
                    }
                }
                _ => {
                    stack.push(token);
                },
            }
        }

        Ok(stack)
    }

    // pub fn process_operator_level<II: IntoIterator<Item = Self>>(tokens: II) -> Result<Vec<Self>, Error> {
    //     let mut stack: Vec<Self> = vec![];

    //     let tokens = tokens.into_iter();

    //     for token in tokens {
    //         match token {
    //             ProcessItem::Verb { name } => {
    //                 if let Some(mut target) = stack.pop() {
    //                     target.apply_modifier(name)?;
    //                 }
    //                 else {
    //                     bail!(ProcessError::Empty);
    //                 }
    //             }
    //             ProcessItem::Annotation { name } => {
    //                 if let Some(mut target) = stack.pop() {
    //                     target.apply_annotation(name)?;
    //                 }
    //                 else {
    //                     bail!(ProcessError::Empty);
    //                 }
    //             }
    //             _ => {
    //                 stack.push(token);
    //             },
    //         }
    //     }

    //     Ok(stack)
    // }
}

#[derive(Debug, Fail, PartialEq, Eq)]
pub enum ProcessError {
    #[fail(display = "stack is empty")]
    Empty,

    #[fail(display = "target item cannot have modifiers applied to it; target: {:?}", target)]
    NotModifiable {
        target: ProcessItem,
    },

    #[fail(display = "target item cannot have annotations applied to it; target: {:?}", target)]
    NotAnnotatable {
        target: ProcessItem,
    },
}

pub type ModifierTerm = String;
pub type AnnotationTerm = String;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Quantity;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Portion {
    Pseudo(String),
    Quantity,
    Fraction(u8, u8),
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////
// // Concrete tokens

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Ingredient(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Tool(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Container(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Appliance(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// ////////////////////////////////////////////////////////////////////////////////////////////////////
// // Operator tokens

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Verb(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Combine(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Transfer(String, Vec<ModifierTerm>, Vec<AnnotationTerm>);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Measure(Quantity);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Take(Portion);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Leave(Portion);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Place;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Remove;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Configure(String);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Meld;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Discard;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Empty;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct TagSet(String);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct TagGet(String);

// ////////////////////////////////////////////////////////////////////////////////////////////////////
// // Meta tokens

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Simultaneous;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Modifier(String);

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Annotation(String);


// ////////////////////////////////////////////////////////////////////////////////////////////////////
// // Derived tokens

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub enum Mixture {
//     Ingredient(Ingredient),
//     Compound(Box<Mixture>, Box<Mixture>),
// }

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub enum Vessel {
//     Container(Container),
//     Appliance(Appliance),
// }

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct System(Vessel, Mixture);

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum ProcessItem {
//     Ingredient(Ingredient),
//     Mixture(Mixture),
//     Tool(Tool),
//     Container(Container),
//     Appliance(Appliance),
//     Vessel(Vessel),
//     System(System),
// }

// impl ProcessItem {
//     pub fn add_modifier(&mut self, modifier: ModifierTerm) -> Result<(), Error> {
//         match self {
//             ProcessItem::Ingredient(ref mut i) => {
//                 i.1.push(modifier);
//                 Ok(())
//             },
//             ProcessItem::Tool(ref mut i) => {
//                 i.1.push(modifier);
//                 Ok(())
//             },
//             ProcessItem::Container(ref mut i) => {
//                 i.1.push(modifier);
//                 Ok(())
//             },
//             ProcessItem::Appliance(ref mut i) => {
//                 i.1.push(modifier);
//                 Ok(())
//             },
//             _ => {
//                 bail!(ProcessError::NotModifiable{target: self.clone()});
//             }
//         }
//     }

//     pub fn add_annotation(&mut self, annotation: AnnotationTerm) -> Result<(), Error> {
//         match self {
//             ProcessItem::Ingredient(ref mut i) => {
//                 i.2.push(annotation);
//                 Ok(())
//             },
//             ProcessItem::Tool(ref mut i) => {
//                 i.2.push(annotation);
//                 Ok(())
//             },
//             ProcessItem::Container(ref mut i) => {
//                 i.2.push(annotation);
//                 Ok(())
//             },
//             ProcessItem::Appliance(ref mut i) => {
//                 i.2.push(annotation);
//                 Ok(())
//             },
//             _ => {
//                 bail!(ProcessError::NotAnnotatable{target: self.clone()});
//             }
//         }
//     }

//     pub fn combine_with_mixture(self, mixture: Mixture) -> Result<ProcessItem, Error> {
//         match self {
//             ProcessItem::Ingredient(i) => {
//                 let as_ingr = Mixture::Ingredient(i);
//                 Ok(ProcessItem::Mixture(Mixture::Compound(box as_ingr, box mixture)))
//             },
//             _ => {
//                 bail!(ProcessError::NotModifiable{target: self.clone()});
//             }
//         }
//     }
// }

// pub trait Modifiable {
//     fn add_modifier<I: Into<String>>(&mut self, modifier: I);
//     fn modifiers(&self) -> &Vec<String>;
// }

// pub trait Annotatable {
//     fn add_annotation<I: Into<String>>(&mut self, annotation: I);
//     fn annotations(&self) -> &Vec<String>;
// }

// pub trait Taggable {
//     fn set_tag<I: Into<String>>(&mut self, tag: I);
//     fn has_tag<I: AsRef<String>>(&self, tag: I) -> bool;
// }

// impl Modifiable for Ingredient {
//     fn add_modifier<I: Into<ModifierTerm>>(&mut self, modifier: I) {
//         self.1.push(modifier.into())
//     }

//     fn modifiers(&self) -> &Vec<ModifierTerm> {
//         &self.1
//     }
// }

// impl Annotatable for Ingredient {
//     fn add_annotation<I: Into<AnnotationTerm>>(&mut self, annotation: I) {
//         self.2.push(annotation.into())
//     }

//     fn annotations(&self) -> &Vec<AnnotationTerm> {
//         &self.2
//     }
// }

// impl System {
//     pub fn split(self) -> (Vessel, Mixture) {
//         return (self.0, self.1)
//     }

//     pub fn add_mixture(&mut self, mixture: Mixture) {
//         let new_mixture = Mixture::Compound(Box::new(self.1.clone()), Box::new(mixture));
//         self.1 = new_mixture;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::Mixture;
//     use super::Ingredient;

//     #[test]
//     fn test_mixture() {
//         let x = Mixture::Ingredient(Ingredient("apple".to_string(), vec![], vec![]));
//     }
// }
