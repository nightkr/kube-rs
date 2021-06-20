pub use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use std::{borrow::Cow, collections::BTreeMap};

/// An accessor trait for a kubernetes Resource.
///
/// This is for a subset of Kubernetes type that do not end in `List`.
/// These types, using [`ObjectMeta`], SHOULD all have required properties:
/// - `.metadata`
/// - `.metadata.name`
///
/// And these optional properties:
/// - `.metadata.namespace`
/// - `.metadata.resource_version`
///
/// This avoids a bunch of the unnecessary unwrap mechanics for apps.
pub trait Resource {
    /// Type information for types that do not know their resource information at compile time.
    ///
    /// Types that know their metadata at compile time should select `DynamicType = ()`.
    /// Types that require some information at runtime should select `DynamicType`
    /// as type of this information.
    ///
    /// See [`DynamicObject`](crate::dynamic::DynamicObject) for a valid implementation of non-k8s-openapi resources.
    type DynamicType: Send + Sync + 'static;

    /// Returns kind of this object
    fn kind(dt: &Self::DynamicType) -> Cow<'_, str>;
    /// Returns group of this object
    fn group(dt: &Self::DynamicType) -> Cow<'_, str>;
    /// Returns version of this object
    fn version(dt: &Self::DynamicType) -> Cow<'_, str>;
    /// Returns apiVersion of this object
    fn api_version(dt: &Self::DynamicType) -> Cow<'_, str> {
        let group = Self::group(dt);
        if group.is_empty() {
            return Self::version(dt);
        }
        let mut group = group.into_owned();
        group.push('/');
        group.push_str(&Self::version(dt));
        group.into()
    }
    /// Returns the plural name of the kind
    ///
    /// This is known as the resource in apimachinery, we rename it for disambiguation.
    fn plural(dt: &Self::DynamicType) -> Cow<'_, str>;

    /// Creates a url path for http requests for this resource
    fn url_path(dt: &Self::DynamicType, namespace: Option<&str>) -> String {
        let n = if let Some(ns) = namespace {
            format!("namespaces/{}/", ns)
        } else {
            "".into()
        };
        let group = Self::group(dt);
        let api_version = Self::api_version(dt);
        let plural = Self::plural(dt);
        format!(
            "/{group}/{api_version}/{namespaces}{plural}",
            group = if group.is_empty() { "api" } else { "apis" },
            api_version = api_version,
            namespaces = n,
            plural = plural
        )
    }

    /// Metadata that all persisted resources must have
    fn meta(&self) -> &ObjectMeta;
    /// Metadata that all persisted resources must have
    fn meta_mut(&mut self) -> &mut ObjectMeta;
}

/// Implement accessor trait for any ObjectMeta-using Kubernetes Resource
impl<K> Resource for K
where
    K: k8s_openapi::Metadata<Ty = ObjectMeta>,
{
    type DynamicType = ();

    fn kind(_: &()) -> Cow<'_, str> {
        K::KIND.into()
    }

    fn group(_: &()) -> Cow<'_, str> {
        K::GROUP.into()
    }

    fn version(_: &()) -> Cow<'_, str> {
        K::VERSION.into()
    }

    fn api_version(_: &()) -> Cow<'_, str> {
        K::API_VERSION.into()
    }

    fn plural(_: &()) -> Cow<'_, str> {
        K::URL_PATH_SEGMENT.into()
    }

    fn meta(&self) -> &ObjectMeta {
        self.metadata()
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        self.metadata_mut()
    }
}

/// Helper methods for resources.
pub trait ResourceExt: Resource {
    /// Returns the name of the resource, panicking if it is
    /// missing. Use this function if you know that name is set, for example
    /// when resource was received from the apiserver.
    /// Because of `.metadata.generateName` field, in other contexts name
    /// may be missing.
    ///
    /// For non-panicking alternative, you can directly read `name` field
    /// on the `self.meta()`.
    fn name(&self) -> String;
    /// The namespace the resource is in
    fn namespace(&self) -> Option<String>;
    /// The resource version
    fn resource_version(&self) -> Option<String>;
    /// Unique ID (if you delete resource and then create a new
    /// resource with the same name, it will have different ID)
    fn uid(&self) -> Option<String>;
    /// Returns resource labels
    fn labels(&self) -> &BTreeMap<String, String>;
    /// Provides mutable access to the labels
    fn labels_mut(&mut self) -> &mut BTreeMap<String, String>;
    /// Returns resource annotations
    fn annotations(&self) -> &BTreeMap<String, String>;
    /// Provider mutable access to the annotations
    fn annotations_mut(&mut self) -> &mut BTreeMap<String, String>;
    /// Returns resource owner references
    fn owner_references(&self) -> &[OwnerReference];
    /// Provides mutable access to the owner references
    fn owner_references_mut(&mut self) -> &mut Vec<OwnerReference>;
    /// Returns resource finalizers
    fn finalizers(&self) -> &[String];
    /// Provides mutable access to the finalizers
    fn finalizers_mut(&mut self) -> &mut Vec<String>;
}

impl<K: Resource> ResourceExt for K {
    fn name(&self) -> String {
        self.meta().name.clone().expect(".metadata.name missing")
    }

    fn namespace(&self) -> Option<String> {
        self.meta().namespace.clone()
    }

    fn resource_version(&self) -> Option<String> {
        self.meta().resource_version.clone()
    }

    fn uid(&self) -> Option<String> {
        self.meta().uid.clone()
    }

    fn labels(&self) -> &BTreeMap<String, String> {
        &self.meta().labels
    }

    fn labels_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.meta_mut().labels
    }

    fn annotations(&self) -> &BTreeMap<String, String> {
        &self.meta().annotations
    }

    fn annotations_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.meta_mut().annotations
    }

    fn owner_references(&self) -> &[OwnerReference] {
        self.meta().owner_references.as_slice()
    }

    fn owner_references_mut(&mut self) -> &mut Vec<OwnerReference> {
        &mut self.meta_mut().owner_references
    }

    fn finalizers(&self) -> &[String] {
        self.meta().finalizers.as_slice()
    }

    fn finalizers_mut(&mut self) -> &mut Vec<String> {
        &mut self.meta_mut().finalizers
    }
}
