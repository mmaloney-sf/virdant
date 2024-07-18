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
pub struct ElementId(ItemId, Ident);

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ComponentId(ModDefId, Ident);


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

pub trait HasItem {
    fn item(&self) -> ItemId;
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

impl std::fmt::Display for ComponentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}::{}", self.item().package(), self.item().name(), self.name())
    }
}

impl std::fmt::Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}::{}", self.item().package(), self.item(), self.name())
    }
}

impl ElementId {
    pub fn item(&self) -> ItemId {
        self.0.clone()
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

impl From<PackageId> for Ident {
    fn from(value: PackageId) -> Self {
        value.0.clone()
    }
}

debug_impl!(PackageId);
debug_impl!(ItemId);
debug_impl!(ElementId);
debug_impl!(ComponentId);

item_id!(ModDef, ModDefId);
item_id!(UnionDef, UnionDefId);
item_id!(StructDef, StructDefId);
item_id!(PortDef, PortDefId);

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
        self.1.clone()
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

impl ElementId {
    pub(crate) fn from_ident(item_id: ItemId, name: Ident) -> Self {
        ElementId(item_id, name)
    }
}

impl ComponentId {
    pub(crate) fn from_ident(moddef_id: ModDefId, name: Ident) -> Self {
        ComponentId(moddef_id, name)
    }
}

impl AsElement for ElementId {
    fn as_element(&self) -> ElementId {
        self.clone()
    }
}

impl AsElement for ComponentId {
    fn as_element(&self) -> ElementId {
        ElementId::from_ident(self.0.as_item(), self.1.clone())
    }
}

impl HasItem for ItemId {
    fn item(&self) -> ItemId {
        self.clone()
    }
}

impl HasItem for ComponentId {
    fn item(&self) -> ItemId {
        self.0.as_item()
    }
}

impl ComponentId {
    pub fn moddef(&self) -> ModDefId {
        self.0.clone()
    }
}

impl Named for ComponentId {
    fn name(&self) -> Ident {
        self.1.clone()
    }
}
