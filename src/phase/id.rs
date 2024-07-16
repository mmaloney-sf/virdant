use super::*;

////////////////////////////////////////////////////////////////////////////////
// Package
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PackageId(Ident);


////////////////////////////////////////////////////////////////////////////////
// Items
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ItemId {
    ModDef(ModDefId),
    UnionDef(UnionDefId),
    StructDef(StructDefId),
    PortDef(PortDefId),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModDefId(PackageId, Ident);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UnionDefId(PackageId, Ident);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StructDefId(PackageId, Ident);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PortDefId(PackageId, Ident);


////////////////////////////////////////////////////////////////////////////////
// Elements
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum ElementId {
    ModDef(ModDefElementId),
    UnionDef(UnionDefElementId),
    StructDef(StructDefElementId),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModDefElementId(ModDefId, Ident);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct UnionDefElementId(UnionDefId, Ident);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StructDefElementId(StructDefId, Ident);


////////////////////////////////////////////////////////////////////////////////
// Traits
////////////////////////////////////////////////////////////////////////////////

pub trait AsItem {
    fn as_item(&self) -> ItemId;
}

pub trait Named {
    fn name(&self) -> Ident;
}

pub trait HasPackage {
    fn package(&self) -> PackageId;
}

pub trait AsElement {
    fn as_element(&self) -> ElementId;
}


////////////////////////////////////////////////////////////////////////////////
// Macros
////////////////////////////////////////////////////////////////////////////////

macro_rules! debug_impl {
    ($name:ident) => {
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }
    };
}

macro_rules! item_id {
    ($ctor:ident, $name:ident) => {
        impl AsItem for $name {
            fn as_item(&self) -> ItemId {
                ItemId::$ctor(self.clone())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}::{}", self.package(), self.name())
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }

        impl Named for $name {
            fn name(&self) -> Ident {
                self.1.clone()
            }
        }

        impl HasPackage for $name {
            fn package(&self) -> PackageId {
                self.0.clone()
            }
        }

        impl $name {
            pub(crate) fn from_ident(package_id: PackageId, name: Ident) -> Self {
                $name(package_id, name)
            }
        }
    }
}

macro_rules! element_id {
    ($ctor:ident, $name:ident, $item:ident, $item_lowercase:ident) => {
        impl AsElement for $name {
            fn as_element(&self) -> ElementId {
                ElementId::$ctor(self.clone())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}::{}", self.package(), self.name())
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{self}")
            }
        }

        impl Named for $name {
            fn name(&self) -> Ident {
                self.1.clone()
            }
        }

        impl HasPackage for $name {
            fn package(&self) -> PackageId {
                self.0.package()
            }
        }

        impl $name {
            pub(crate) fn from_ident(item_id: $item, name: Ident) -> Self {
                $name(item_id, name)
            }

            pub fn $item_lowercase(&self) -> $item {
                self.0.clone()
            }

            pub fn item(&self) -> ItemId {
                self.0.as_item()
            }
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// Impls
////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for ItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.package(), self.name())
    }
}

impl std::fmt::Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}::{}", self.item().package(), self.item(), self.name())
    }
}

impl ElementId {
    pub fn item(&self) -> ItemId {
        match self {
            ElementId::ModDef(component) => component.item(),
            ElementId::UnionDef(alt) => alt.item(),
            ElementId::StructDef(field) => field.item(),
        }
    }
}

impl HasPackage for ItemId {
    fn package(&self) -> PackageId {
        match self {
            ItemId::ModDef(moddef) => moddef.0.clone(),
            ItemId::UnionDef(uniondef) => uniondef.0.clone(),
            ItemId::StructDef(structdef) => structdef.0.clone(),
            ItemId::PortDef(portdef) => portdef.0.clone(),
        }
    }
}

debug_impl!(PackageId);
debug_impl!(ItemId);
debug_impl!(ElementId);

item_id!(ModDef, ModDefId);
item_id!(UnionDef, UnionDefId);
item_id!(StructDef, StructDefId);
item_id!(PortDef, PortDefId);

element_id!(ModDef, ModDefElementId, ModDefId, moddef);
element_id!(UnionDef, UnionDefElementId, UnionDefId, uniondef);
element_id!(StructDef, StructDefElementId, StructDefId, structdef);

impl Named for ItemId {
    fn name(&self) -> Ident {
        match self {
            ItemId::ModDef(moddef) => moddef.name(),
            ItemId::UnionDef(uniondef) => uniondef.name(),
            ItemId::StructDef(structdef) => structdef.name(),
            ItemId::PortDef(portdef) => portdef.name(),
        }
    }
}

impl Named for ElementId {
    fn name(&self) -> Ident {
        match self {
            ElementId::ModDef(component) => component.name(),
            ElementId::UnionDef(alt) => alt.name(),
            ElementId::StructDef(field) => field.name(),
        }
    }
}

impl Named for PackageId {
    fn name(&self) -> Ident {
        self.0.clone()
    }
}

impl PackageId {
    pub(crate) fn from_ident(package: Ident) -> Self {
        PackageId(package)
    }
}
