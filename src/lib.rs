//! Lnk plugin take a VFile attribute return from a node and  add the result of an lnk function to the attribute of this node

use std::sync::Arc;
use std::fmt::Debug;

use tap::plugin;
use tap::vfile::VFile;
use tap::value::Value;
use tap::config_schema;
use tap::error::RustructError;
use tap::reflect::ReflectStruct;
use tap::tree::{TreeNodeId, TreeNodeIdSchema};
use tap::plugin::{PluginInfo, PluginInstance, PluginConfig, PluginArgument, PluginResult, PluginEnvironment};

use serde::{Serialize, Deserialize};
use schemars::{JsonSchema};
use tap_derive::Reflect;

plugin!("lnk", "Windows", "Parse lnk file", LnkPlugin, Arguments);

#[derive(Debug, Serialize, Deserialize,JsonSchema)]
pub struct Arguments
{
  #[schemars(with = "TreeNodeIdSchema")] 
  file : TreeNodeId,
}

#[derive(Debug, Serialize, Deserialize,Default)]
pub struct Results
{
}

#[derive(Default)]
pub struct LnkPlugin
{
}

impl LnkPlugin
{
  fn run(&mut self, args : Arguments, env : PluginEnvironment) -> anyhow::Result<Results>
  {
    let file_node = env.tree.get_node_from_id(args.file).ok_or(RustructError::ArgumentNotFound("file"))?;
    file_node.value().add_attribute(self.name(), None, None); 
    let data = file_node.value().get_value("data").ok_or(RustructError::ValueNotFound("data"))?;
    let data_builder = data.try_as_vfile_builder().ok_or(RustructError::ValueTypeMismatch)?;
    let mut file = data_builder.open()?;

    let lnk = match Lnk::from_file(&mut file)
    {
       Ok(lnk) => lnk,
       Err(err) => { file_node.value().add_attribute(self.name(), None, None); return Err(err) },
    };
      
    file_node.value().add_attribute("lnk", Arc::new(lnk), None);

    Ok(Results{})
  }
}


#[derive(Debug, Reflect)]
pub struct Lnk
{
  shell : Arc<ShellLinkHeader>, //store here directly ?
  path : Arc<StringData>,
  //link_target_id_list : Arc<LinkTargetIdList>, seems used only to seek
  info : Arc<LinkInfo>,
  //extra_data : Arc<ExtraData>,
}

impl Lnk
{
  pub fn new(shell: ShellLinkHeader, 
             path : StringData,
             info : LinkInfo, 
             //extra_data : ExtraData
            ) -> Self
  {
    Lnk{ shell: Arc::new(shell), 
         path : Arc::new(path), 
         info : Arc::new(info),
         //extra_data : Arc::new(extra_data) 
       }
  }

  pub fn from_file<T : VFile>(mut file : &mut T) -> anyhow::Result<Lnk>
  {
    let lnk = parselnk::Lnk::new(&mut file)?;
    Ok(Lnk::new(ShellLinkHeader{ inner : lnk.header}, 
                StringData {inner : lnk.string_data},
                LinkInfo{ inner : lnk.link_info}))
                //ExtraData{ inner : lnk.extra_data}))
  } 
}

#[derive(Debug)]
pub struct ShellLinkHeader
{
  inner : parselnk::ShellLinkHeader,
}


impl ReflectStruct for ShellLinkHeader
{
  fn name(&self) -> &'static str
  {
    "ShellLinkHeader"
  }

  fn infos(&self) -> Vec<(&'static str, Option<&'static str>) >
  {
    vec![("created", None), ("modified", None), ("acccessed", None), ("attributes", None), ("size", None)]
  }

  fn get_value(&self, name : &str) -> Option<Value>
  {
    match name
    {
      "created" => Some(Value::from(self.inner.created_on.unwrap())),
      "modified" => Some(Value::from(self.inner.modified_on.unwrap())),
      "accessed" => Some(Value::DateTime(self.inner.accessed_on.unwrap())),
      "attributes" => Some(Value::from(format!("{:?}", self.inner.file_attributes))),
      "size" => Some(Value::from(self.inner.file_size)), 
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct StringData
{
  //[reflect(Deref(inner type == ...))]
  inner : parselnk::StringData,
}

impl ReflectStruct for StringData
{
  fn name(&self) -> &'static str
  {
    "StringData"
  }

  fn infos(&self) -> Vec<(&'static str, Option<&'static str>) >
  {
    vec![("name", None), ("relative_path", None), ("working_dir", None), ("command_line_arguments", None), ("icon_location", None)]
  }

  fn get_value(&self, name : &str) -> Option<Value>
  {
    match name
    {
      "name" => self.inner.name_string.as_ref().map(|value| Value::String(value.clone())),
      "relative_path" => self.inner.relative_path.as_ref().map(|value| Value::String(value.display().to_string())),
      "working_dir" => self.inner.working_dir.as_ref().map(|value| Value::String(value.display().to_string())),
      "command_line_arguments" => self.inner.command_line_arguments.as_ref().map(|value| Value::String(value.clone())),
      "icon_location" => self.inner.icon_location.as_ref().map(|value| Value::String(value.display().to_string())),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct LinkInfo
{
  inner : parselnk::LinkInfo,
}

impl ReflectStruct for LinkInfo 
{
  fn name(&self) -> &'static str
  {
    "LinkInfo"
  }

  fn infos(&self) -> Vec<(&'static str, Option<&'static str>) >
  {
    vec![("flags", None), ("local_base_path", None), ("common_path_suffix", None), ("local_base_path_unicode", None), ("common_path_suffix_unicode", None)]
  }

  fn get_value(&self, name : &str) -> Option<Value>
  {
    match name
    {
      "flags" => self.inner.link_info_flags.as_ref().map(|value| Value::String(format!("{:?}", value))),
      //XXX switch between unicode and non unicode version 
      "local_base_path" => self.inner.local_base_path.as_ref().map(|value| Value::String(value.clone())),
      "common_path_suffix" => self.inner.common_path_suffix.as_ref().map(|value| Value::String(value.clone())),
      "local_base_path_unicode" => self.inner.local_base_path_unicode.as_ref().map(|value| Value::String(value.clone())),
      "common_path_suffix_unicode" => self.inner.common_path_suffix_unicode.as_ref().map(|value| Value::String(value.clone())),
      _ => None,
    }
  }
}

/*#[derive(Debug, Reflect)]
pub struct ExtraData
{
  #[reflect(skip)]
  inner : parselnk::ExtraData,
  //console_props: Option<ConsoleDataBlock>,
  //console_fe_props: Option<ConsoleFEDataBlock>,
  //shim_props: Option<ShimDataBlock>,
}*/
